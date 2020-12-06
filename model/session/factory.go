package session

import (
	"crypto/rand"
	"encoding/base64"
	"fmt"
	"io"
	"sync"
	"time"

	"github.com/alvidir/tp-auth/model/client"
	pb "github.com/alvidir/tp-auth/proto/client"
)

// All session instances
var (
	allInstancesByClientID = &sync.Map{}
	fromCookieToClientID   = &sync.Map{}
)

type sessionID string
type clientID int64

func registerSession(session *session) (err error) {
	sid := sessionID(session.GetCookie())
	cid := clientID(session.GetID())

	if _, exists := allInstancesByClientID.Load(sid); exists {
		return fmt.Errorf(errCookieAlreadyExists, sid)
	}

	allInstancesByClientID.Store(cid, session)
	fromCookieToClientID.Store(sid, cid)
	return
}

func removeSession(cid clientID) (err error) {
	content, exists := allInstancesByClientID.Load(cid)
	if !exists {
		return fmt.Errorf(errClientNotExists, cid)
	}

	if _, ok := content.(*session); !ok {
		return fmt.Errorf(errAssertionFailed, cid)
	}

	allInstancesByClientID.Delete(cid)
	return
}

func removeCookie(sid sessionID) (cid clientID, err error) {
	content, exists := fromCookieToClientID.Load(sid)
	if !exists {
		err = fmt.Errorf(errCookieNotExists, sid)
		return
	}

	cid, ok := content.(clientID)
	if !ok {
		err = fmt.Errorf(errAssertionFailed, sid)
		return
	}

	fromCookieToClientID.Delete(sid)
	return
}

func remove(sid sessionID) (err error) {
	var cid clientID
	if cid, err = removeCookie(sid); err != nil {
		return
	}

	return removeSession(cid)
}

// RandomString returns a pseudo-random string of 32 bits
func RandomString() (str string, err error) {
	b := make([]byte, 32)
	if _, err = io.ReadFull(rand.Reader, b); err != nil {
		return
	}

	str = base64.URLEncoding.EncodeToString(b)
	return
}

// NewSession returns a brand new session for the provided client
func NewSession(client client.Controller, timeout time.Duration) (ctrl Controller, err error) {
	deadline := time.Now().Add(timeout)
	if time.Now().Unix() < deadline.Unix() {
		err = fmt.Errorf(errNoDeadline)
		return
	}

	var cookie string
	if cookie, err = RandomString(); err != nil {
		return
	}

	session := &session{
		Controller: client,
		CreatedAt:  time.Now(),
		TouchAt:    time.Now(),
		Cookie:     cookie,
		Deadline:   deadline,
		Status:     pb.Status_NEW,
	}

	err = registerSession(session)
	ctrl = session
	return
}

// KillSession logs out the session with the provided cookie
func KillSession(cookie string) error {
	sid := sessionID(cookie)
	return remove(sid)
}

// FindSessionByClient returns the session with the provided cookie, if exists
func FindSessionByClient(id int64) (ctrl Controller, err error) {
	cid := clientID(id)

	content, exists := allInstancesByClientID.Load(cid)
	if !exists {
		err = fmt.Errorf(errClientNotExists, cid)
		return
	}

	var ok bool
	if ctrl, ok = content.(*session); !ok {
		err = fmt.Errorf(errAssertionFailed, cid)
	}

	return
}

// FindSessionByCookie returns the session with the provided cookie, if exists
func FindSessionByCookie(cookie string) (ctrl Controller, err error) {
	sid := sessionID(cookie)

	content, exists := fromCookieToClientID.Load(sid)
	if !exists {
		err = fmt.Errorf(errCookieNotExists, cookie)
		return
	}

	cid, ok := content.(clientID)
	if !ok {
		err = fmt.Errorf(errAssertionFailed, cookie)
	}

	return FindSessionByClient(int64(cid))
}
