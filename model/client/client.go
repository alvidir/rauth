package client

import (
	"fmt"
	"sync"
	"time"

	"github.com/alvidir/tp-auth/model/credential"
	"gorm.io/gorm"
)

// A Client represents some client in the system
type Client struct {
	gorm.Model
	ID          uint                              `json:"id" gorm:"primaryKey; autoIncrement:true"`
	PWD         string                            `json:"name" gorm:"not null"`
	Status      Status                            `json:"status" gorm:"not null"`
	CreatedAt   time.Time                         `json:"createdAt"`
	UpdatedAt   time.Time                         `json:"updatedAt"`
	Credentials map[string]*credential.Credential `json:"credentials" gorm:"foreignKey:ID"`
	OwnerID     int                               `json:"owner_id"`
	OwnerType   string                            `json:"owner_type"`
	extension   Extension
	mu          sync.RWMutex
}

func (client *Client) credentialExists(name string) (ok bool) {
	client.mu.RLock()
	defer client.mu.RUnlock()

	_, ok = client.Credentials[name]
	return
}

// SetExtension sets an extension to the client if, and only if, no one has been set earlier
func (client *Client) SetExtension(ext Extension) Extension {
	if client.extension == nil {
		client.mu.Lock()
		defer client.mu.Unlock()

		if client.extension == nil {
			client.extension = ext
		}
	}

	return client.extension
}

// GetStatus returns the client status
func (client *Client) GetStatus() string {
	return client.Status.String()
}

// MatchPassword returns true if, and only if, the provided hash do match with the pqssword's one
func (client *Client) MatchPassword(pwd string) bool {
	return pwd == client.PWD
}

// SetCredential sets a new credential to the client
func (client *Client) SetCredential(cred *credential.Credential) error {
	if ok := client.credentialExists(cred.GetName()); ok {
		return fmt.Errorf(errCredentialAlreadyExists, cred.GetName())
	}

	client.mu.Lock()
	defer client.mu.Unlock()
	client.Credentials[cred.GetName()] = cred
	return nil
}
