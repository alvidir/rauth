package transactions

import (
	"context"
	"log"

	pb "github.com/alvidir/tp-auth/proto/session"
)

// TxSignup represents an
type TxSignup struct {
	req *pb.SignupRequest
}

// Precondition validates the transaction is ready to run
func (tx *TxSignup) Precondition() error {
	return nil
}

// Postcondition creates new user and a opens its first session
func (tx *TxSignup) Postcondition(context.Context) (interface{}, error) {
	log.Printf("Got a Signup request")
	return nil, nil
}

// Commit commits the transaction result
func (tx *TxSignup) Commit() error {
	return nil
}

// Rollback rollbacks any change caused while the transaction
func (tx *TxSignup) Rollback() {

}
