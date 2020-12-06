package client

import (
	"sync"
	"time"
)

// A Client represents some client in the system
type Client struct {
	ID        int64      `xorm:"pk autoincr"`                                                   //`json:"id" gorm:"primaryKey; autoIncrement:true"`
	Name      string     `xorm:"varchar(16) not null unique 'client_name' comment('NickName')"` //`json:"name" gorm:"not null; unique"`
	PWD       string     `xorm:"varchar(32) not null 'client_pwd'"`                             //`json:"-" gorm:"password"`
	Status    Status     `xorm:"smallint not null 'client_status'"`                             //`json:"status" gorm:"status"`
	CreatedAt time.Time  `xorm:"created"`                                                       //`json:"created_at" gorm:"created_at"`
	UpdatedAt time.Time  `xorm:"updated"`                                                       //`json:"updated_at" gorm:"updated_at"`
	Creds     []string   //`json:"-" gorm:"credentials"`
	extension Extension  `xorm:"-"`
	mu        sync.Mutex `xorm:"-"`
}

// GetAddr returns the client addr
func (client *Client) GetAddr() string {
	return client.extension.GetAddr()
}

// GetID returns the client addr
func (client *Client) GetID() int64 {
	return client.ID
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

// ClientStatus returns the client status
func (client *Client) ClientStatus() Status {
	return client.Status
}

// MatchPassword returns true if, and only if, the provided hash do match with the pqssword's one
func (client *Client) MatchPassword(pwd string) bool {
	return pwd == client.PWD
}
