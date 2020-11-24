package client

// A Controller represents a registered client
type Controller interface {
	GetStatus() string
	MatchPassword(string) bool
}
