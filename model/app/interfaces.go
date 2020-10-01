package app

import (
	"github.com/alvidir/tp-auth/model/client"
)

// A App represents an App client
type App interface {
	client.Extension
}
