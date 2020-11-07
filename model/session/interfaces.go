package session

// A Controller represents the session of some client
type Controller interface {
	GetCookieName() string
	GetCookieValue() string
}
