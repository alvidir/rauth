package transactions

import (
	"github.com/alvidir/util/pattern/transaction"
)

// NewTxLogin builds a brand new instance of TxLogin
func NewTxLogin() transaction.Tx {
	body := &TxLogin{}
	return transaction.NewTransaction(body)
}

// NewTxGoogleLogin builds a brand new instance of TxGoogleLogin
func NewTxGoogleLogin() transaction.Tx {
	body := &TxGoogleLogin{}
	return transaction.NewTransaction(body)
}

// NewTxLogout builds a brand new instance of TxLogout
func NewTxLogout() transaction.Tx {
	body := &TxLogout{}
	return transaction.NewTransaction(body)
}

// NewTxSignup builds a brand new instance of TxSignup
func NewTxSignup() transaction.Tx {
	body := &TxSignup{}
	return transaction.NewTransaction(body)
}
