package session

import (
	"github.com/alvidir/mastermind/service/session/transactions"
	"github.com/alvidir/util/pattern/transaction"
)

func newTxLogin() transaction.Tx {
	body := &transactions.TxLogin{}
	return transaction.NewTransaction(body)
}

func newTxGoogleLogin() transaction.Tx {
	body := &transactions.TxGoogleLogin{}
	return transaction.NewTransaction(body)
}

func newTxLogout() transaction.Tx {
	body := &transactions.TxLogout{}
	return transaction.NewTransaction(body)
}

func newTxSignup() transaction.Tx {
	body := &transactions.TxSignup{}
	return transaction.NewTransaction(body)
}
