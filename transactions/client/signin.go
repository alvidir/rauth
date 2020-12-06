package client

import (
	"context"
	"crypto/sha1"
	"encoding/base64"
	"log"
	"time"

	"github.com/alvidir/tp-auth/mysql"
	"xorm.io/xorm"

	"github.com/alvidir/tp-auth/model/client"
	"github.com/alvidir/tp-auth/model/user"

	"github.com/alvidir/tp-auth/google"
	"github.com/alvidir/tp-auth/model/session"
	pb "github.com/alvidir/tp-auth/proto/client"
	"google.golang.org/api/oauth2/v2"
)

// TxGoogleSignin represents an
type TxGoogleSignin struct {
	req  *pb.GoogleSigninRequest
	info *oauth2.Tokeninfo
	sess session.Controller
	new  user.Controller
}

func (tx *TxGoogleSignin) renderProtobuffer(ctrl session.Controller) *pb.SessionResponse {
	return &pb.SessionResponse{
		Cookie:   ctrl.GetCookie(),
		Deadline: ctrl.GetDeadline().Unix(),
		Status:   ctrl.SessionStatus(),
		Token:    "ephimeral token",
	}
}

func (tx *TxGoogleSignin) buildNewClientUser() (err error) {
	var random string
	if random, err = session.RandomString(); err != nil {
		return
	}

	hasher := sha1.New()
	if _, err = hasher.Write([]byte(random)); err != nil {
		return
	}

	sha := base64.URLEncoding.EncodeToString(hasher.Sum(nil))
	client := client.New(tx.info.UserId, sha)
	tx.new = user.New(client, tx.info.Email)
	return nil
}

// Precondition validates the transaction is ready to run. That means it does validates all parameters and
// connection requirements to make sure the transaction has chances of commit.
func (tx *TxGoogleSignin) Precondition() (err error) {
	tx.info, err = google.VerifyTokenID(tx.req.TokenID)
	return
}

// Postcondition creates a new session or update the latest one for a provided user, if exists.
func (tx *TxGoogleSignin) Postcondition(ctx context.Context) (resp interface{}, err error) {
	log.Printf("Got a Signin request for client %s", tx.info.Email)

	// SIGNUP //
	var ctrl user.Controller
	if ctrl, err = user.FindUserByEmail(ctx, tx.info.Email); err != nil {
		log.Printf("Signing up a new client %s", tx.info.Email)
		if err = tx.buildNewClientUser(); err != nil {
			return
		}

		ctrl = tx.new
	}

	// SESSION //
	var sess session.Controller
	if sess, err = session.FindSessionByClient(ctrl.GetID()); err == nil {
		log.Printf("The session for %s already exists", tx.info.Email)
		resp = tx.renderProtobuffer(sess)
		return
	}

	// LOGIN //
	log.Printf("Logging in client %s", ctrl.GetAddr())
	timeout := time.Duration(tx.info.ExpiresIn)
	if sess, err = session.NewSession(ctrl, timeout); err != nil {
		return
	}

	response := tx.renderProtobuffer(sess)
	log.Printf("Got a cookie %s for client %v", response.Cookie, sess.GetAddr())
	return response, nil
}

// Commit commits the session and make its alive.
func (tx *TxGoogleSignin) Commit() (err error) {
	var engine *xorm.Engine
	if engine, err = mysql.OpenStream(); err != nil {
		return
	}

	defer engine.Close()
	if tx.new != nil {
		if err = engine.Sync2(&user.User{}); err != nil {
			return
		}
		_, err = engine.Insert(tx.new)
		log.Println(tx.new.GetID())
		return
	}

	return
}

// Rollback the session in order to make it non existence
func (tx *TxGoogleSignin) Rollback() {

}
