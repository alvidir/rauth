package transactions

import (
	"github.com/alvidir/util/pattern/transaction"
)

func newTxLogin() transaction.Tx {
	body := &TxLogin{}
	return transaction.NewTransaction(body)
}

func newTxGoogleLogin() transaction.Tx {
	body := &TxGoogleLogin{}
	return transaction.NewTransaction(body)
}

func newTxLogout() transaction.Tx {
	body := &TxLogout{}
	return transaction.NewTransaction(body)
}

func newTxSignup() transaction.Tx {
	body := &TxSignup{}
	return transaction.NewTransaction(body)
}
