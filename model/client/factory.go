package client

import (
	"time"
)

// NewClient builds a brand new client with a provided password
func NewClient(kind Extension, pwd string) *Client {
	return &Client{
		PWD:       pwd,
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
	}
}
