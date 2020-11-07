package credential

// A Controller represents a public ssh key
type Controller interface {
	GetPublic() string
}
