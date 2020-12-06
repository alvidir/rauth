package client

import (
	"time"
)

// New builds a brand new client with a provided password
func New(name, pwd string) Controller {
	return &Client{
		Name:      name,
		PWD:       pwd,
		CreatedAt: time.Now(),
		UpdatedAt: time.Now(),
		Status:    PENDING,
	}
}
