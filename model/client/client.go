package client

import (
	"sync"
	"time"
)

// A Client represents some client in the system
type Client struct {
	ID        uint      `json:"id" bson:"_id,omitempty"`
	PWD       string    `json:"name" bson:"password"`
	Status    Status    `json:"status" bson:"status"`
	CreatedAt time.Time `json:"created_at" bson:"created_at"`
	UpdatedAt time.Time `json:"updated_at" bson:"updated_at"`
	Creds     []string  `json:"-" bson:"credentials"`
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
