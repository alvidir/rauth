package provider

import (
	"sync"
	"time"

	"github.com/alvidir/tp-auth/model/client"
	"github.com/alvidir/tp-auth/model/session"
)

type sessionName string
type sessionValue *session.Session

type provider struct {
	ID       string        `json:"id" bson:"_id,omitempty"` // provider id
	Name     string        `json:"name" bson:"name"` // provider name
	Prefix   string        `json:"prefix" bson:"prefix"` // prefix to append at front of any cookie
	Timeout  time.Duration `json:"timeout" bson:"timeout"` // how long any session of this provider can be alive
	sessions sync.Map 
}

func (provider *provider) SetPrefix(prefix string) {
	provider.Prefix = prefix
}

func (provider *provider) GetName() string {
	return provider.Name
}

func (provider *provider) NewSession(string, *client.Client) (s *session.Session, err error) {
	return
}

func (provider *provider) GetSession(string) (s *session.Session, err error) {
	return
}

func (provider *provider) DestroySession(string) (err error) {
	return
}

func (provider *provider) Purge(time.Time) (howmany int) {
	return
}
