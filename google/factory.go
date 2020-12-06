package google

import (
	"net/http"

	"google.golang.org/api/oauth2/v2"
)

var httpClient = &http.Client{}

// VerifyTokenID ensures the provided token ID is valid
func VerifyTokenID(idToken string) (info *oauth2.Tokeninfo, err error) {
	var oauth2Service *oauth2.Service
	if oauth2Service, err = oauth2.New(httpClient); err != nil {
		return
	}

	tokenInfoCall := oauth2Service.Tokeninfo()
	tokenInfoCall = tokenInfoCall.IdToken(idToken)
	return tokenInfoCall.Do()
}
