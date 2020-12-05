package client

import (
	"sync"
	"time"
)

// A Client represents some client in the system
type Client struct {
	ID        uint      `json:"id" gorm:"primaryKey; autoIncrement:true"`
	PWD       string    `json:"-" gorm:"password"`
	Status    Status    `json:"status" gorm:"status"`
	CreatedAt time.Time `json:"created_at" gorm:"created_at"`
	UpdatedAt time.Time `json:"updated_at" gorm:"updated_at"`
	Creds     []string  `json:"-" gorm:"credentials"`
	extension Extension
	mu        sync.Mutex
}

// SetExtension sets an extension to the client
func (client *Client) SetExtension(ext Extension) bool {
	if client.extension == nil {
		client.mu.Lock()
		defer client.mu.Unlock()

		if client.extension == nil {
			client.extension = ext
		}
	}

	return client.extension == ext
}

// GetStatus returns the client status
func (client *Client) GetStatus() string {
	return client.Status.String()
}

// MatchPassword returns true if, and only if, the provided hash do match with the pqssword's one
func (client *Client) MatchPassword(pwd string) bool {
	return pwd == client.PWD
}
