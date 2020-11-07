package client

// A Extension represents a kind of client that extends from Client
type Extension interface {
}

// A Controller represents a registered client
type Controller interface {
	GetStatus() string
	MatchPassword(string) bool
	SetExtension(Extension) Extension
	SetCredential(...string) error
}
