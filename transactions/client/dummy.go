package client

import (
	"context"
	"time"

	"google.golang.org/api/oauth2/v2"
)

func newDummyTokenInfo() *oauth2.Tokeninfo {
	return &oauth2.Tokeninfo{
		Email:     "testing@gmail.com",
		ExpiresIn: -1,
		UserId:    "1234",
	}
}

// SetupDummyUser inits a dummy user for testing
func SetupDummyUser() (err error) {
	dummy := &TxGoogleSignin{
		info: newDummyTokenInfo(),
	}

	ctx, cancel := context.WithTimeout(context.TODO(), 10*time.Second)
	defer cancel()

	if _, err = dummy.Postcondition(ctx); err != nil {
		return
	}

	return dummy.Commit()
}
