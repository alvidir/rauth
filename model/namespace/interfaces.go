package namespace

// Controller of the namespace
type Controller interface {
	Read(string) (interface{}, error)
	Write(string, interface{}) error
}
