package client

import (
	"time"
)

// New builds a brand new client with a provided password
func New(kind Extension, pwd string) *Client {
	return &Client{
		PWD:       pwd,
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
		extension: kind,
	}
}
