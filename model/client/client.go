package client

import (
	"time"
)

type client struct {
	Extension
	id       string
	password string
	status   string
	creation time.Time
}

func (client *client) GetID() string {
	return client.id
}

func (client *client) GetStatus() string {
	return client.status
}

func (client *client) IsPassword(pwd string) bool {
	return pwd == client.password
}
