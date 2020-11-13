package client

import (
	"sync"
	"time"

	"gorm.io/gorm"
)

// A Client represents some client in the system
type Client struct {
	gorm.Model
	ID        uint      `json:"id" gorm:"primaryKey; autoIncrement:true"`
	PWD       string    `json:"name" gorm:"not null"`
	Status    Status    `json:"status" gorm:"not null"`
	CreatedAt time.Time `json:"created_at"`
	UpdatedAt time.Time `json:"updated_at"`
	OwnerID   uint      `json:"-" gorm:"not null; unique"`
	OwnerType string    `json:"-" gorm:"not null"`
	Creds     []string  `json:"-"`
	owner     Owner
	mu        sync.Mutex
}

// GetStatus returns the client status
func (client *Client) GetStatus() string {
	return client.Status.String()
}

// MatchPassword returns true if, and only if, the provided hash do match with the pqssword's one
func (client *Client) MatchPassword(pwd string) bool {
	return pwd == client.PWD
}

// SetOwner sets the owner of the client (user or app) if is not already set
func (client *Client) SetOwner(owner Owner) Owner {
	if client.owner == nil {
		client.mu.Lock()
		defer client.mu.Unlock()

		if client.owner == nil {
			client.owner = owner
		}
	}

	return owner
}
