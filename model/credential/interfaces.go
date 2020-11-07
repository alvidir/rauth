package credential

// A Controller represents a public ssh key
type Controller interface {
	GetName() string
	GetPublic() string
}
