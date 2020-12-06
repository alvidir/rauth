package client

import (
	"context"
	"log"

	pb "github.com/alvidir/tp-auth/proto/client"
)

// TxLogin represents an
type TxLogin struct {
	req *pb.LoginRequest
}

// Precondition validates the transaction is ready to run. That means it does validates all parameters and
// connection requirements to make sure the transaction has chances of commit.
func (tx *TxLogin) Precondition() error {
	return nil
}

// Postcondition creates a new session or update the latest one for a provided user, if exists.
func (tx *TxLogin) Postcondition(context.Context) (interface{}, error) {
	log.Printf("Got a Login request")
	return nil, nil
}

// Commit commits the session and make its alive.
func (tx *TxLogin) Commit() error {
	return nil
}

// Rollback the session in order to make it non existence
func (tx *TxLogin) Rollback() {

}
