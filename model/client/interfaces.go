package client

// A Owner represents a kind of client that extends from Client
type Owner interface {
}

// A Controller represents a registered client
type Controller interface {
	GetStatus() string
	MatchPassword(string) bool
	SetOwner(Owner) Owner
}
