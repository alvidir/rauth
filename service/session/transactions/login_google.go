package transactions

import (
	"context"
	"log"
)

// TxGoogleLogin represents an
type TxGoogleLogin struct {
}

// Precondition validates the transaction is ready to run. That means it does validates all parameters and
// connection requirements to make sure the transaction has chances of commit.
func (tx *TxGoogleLogin) Precondition() error {
	return nil
}

// Postcondition creates a new session or update the latest one for a provided user, if exists.
func (tx *TxGoogleLogin) Postcondition(context.Context) (interface{}, error) {
	log.Printf("Got a Google login request")
	return nil, nil
}

// Commit commits the session and make its alive.
func (tx *TxGoogleLogin) Commit() error {
	return nil
}

// Rollback the session in order to make it non existence
func (tx *TxGoogleLogin) Rollback() {

}
