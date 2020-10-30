package client

import (
	"time"
)

// NewClient builds a brand new client with a provided password
func NewClient(ext Extension, pwd string) Client {
	return &client{
		Extension: ext,
		password:  pwd,
		creation:  time.Now(),
	}
}
