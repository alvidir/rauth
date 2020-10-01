package credential

// A Credential represents a public ssh key
type Credential interface {
	GetPublic() string
}
