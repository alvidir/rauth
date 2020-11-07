package transactions

import (
	pb "github.com/alvidir/tp-auth/proto/session"
	"github.com/alvidir/util/pattern/transaction"
)

// NewTxSignup builds a brand new instance of TxSignup
func NewTxSignup(req *pb.SignupRequest) transaction.Tx {
	body := &TxSignup{req}
	return transaction.NewTransaction(body)
}

// NewTxLogin builds a brand new instance of TxLogin
func NewTxLogin(req *pb.LoginRequest) transaction.Tx {
	body := &TxLogin{req}
	return transaction.NewTransaction(body)
}

// NewTxGoogleSignin builds a brand new instance of TxGoogleLogin
func NewTxGoogleSignin(req *pb.GoogleSigninRequest) transaction.Tx {
	body := &TxGoogleSignin{req}
	return transaction.NewTransaction(body)
}

// NewTxLogout builds a brand new instance of TxLogout
func NewTxLogout(req *pb.LogoutRequest) transaction.Tx {
	body := &TxLogout{req}
	return transaction.NewTransaction(body)
}
