package session

// A Controller represents the session of some client
type Controller interface {
	MatchCookie(string) bool
}
