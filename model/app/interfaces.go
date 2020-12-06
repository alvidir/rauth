package app

import "github.com/alvidir/tp-auth/model/client"

// A Controller represents an App client
type Controller interface {
	client.Controller
	GetDescription() string
	GetName() string
	GetURI() string
}
