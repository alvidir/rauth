package transactions

import (
	"context"
	"log"

	pb "github.com/alvidir/tp-auth/proto/client"
)

// TxLogout represents an
type TxLogout struct {
	req *pb.LogoutRequest
}

// Precondition validates the transaction is ready to run. That means making sure the session exists.
func (tx *TxLogout) Precondition() error {
	return nil
}

// Postcondition kills the provided session and all them related to the same user
func (tx *TxLogout) Postcondition(context.Context) (interface{}, error) {
	log.Printf("Got a Logout request")
	return nil, nil
}

// Commit commits the logout and makes @sure_of_done
func (tx *TxLogout) Commit() error {
	return nil
}

// Rollback rollbacks the logout keeping all sessions in the latest state
func (tx *TxLogout) Rollback() {

}
