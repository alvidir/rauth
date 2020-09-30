package transactions

import (
	"github.com/alvidir/util/pattern/transaction"
)

func NewTxLogin() transaction.Tx {
	body := &TxLogin{}
	return transaction.NewTransaction(body)
}

func NewTxGoogleLogin() transaction.Tx {
	body := &TxGoogleLogin{}
	return transaction.NewTransaction(body)
}

func NewTxLogout() transaction.Tx {
	body := &TxLogout{}
	return transaction.NewTransaction(body)
}

func NewTxSignup() transaction.Tx {
	body := &TxSignup{}
	return transaction.NewTransaction(body)
}
