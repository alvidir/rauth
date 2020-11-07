package provider

import (
	"time"

	"github.com/alvidir/tp-auth/model/client"
	"github.com/alvidir/tp-auth/model/session"
)

type provider struct {
	ID       string        `json:"id" bson:"_id,omitempty"`
	Name     string        `json:"name" bson:"name"`
	Prefix   string        `json:"prefix" bson:"prefix"`
	AutoGen  bool          `json:"auto" bson:"auto"`
	Timeout  time.Duration `json:"timeout" bson:"timeout"`
	sessions map[string]session.Session
}

func (provider *provider) GetName() string {
	return provider.Name
}

func (provider *provider) NewSession(string, client.Client) (s *session.Session, err error) {
	return
}

func (provider *provider) GetSession(string) (s *session.Session, err error) {
	return
}

func (provider *provider) DestroySession(string) (err error) {
	return
}

func (provider *provider) Purge(int64) (howmany int) {
	return
}
