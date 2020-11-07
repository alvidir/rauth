package transactions

import (
	"context"
	"log"

	pb "github.com/alvidir/tp-auth/proto/session"
)

// TxGoogleSignin represents an
type TxGoogleSignin struct {
	req *pb.GoogleSigninRequest
}

// Precondition validates the transaction is ready to run. That means it does validates all parameters and
// connection requirements to make sure the transaction has chances of commit.
func (tx *TxGoogleSignin) Precondition() error {
	return nil
}

// Postcondition creates a new session or update the latest one for a provided user, if exists.
func (tx *TxGoogleSignin) Postcondition(context.Context) (interface{}, error) {
	log.Printf("Got a Google login request")
	return nil, nil
}

// Commit commits the session and make its alive.
func (tx *TxGoogleSignin) Commit() error {
	return nil
}

// Rollback the session in order to make it non existence
func (tx *TxGoogleSignin) Rollback() {

}
