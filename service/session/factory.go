package session

// ImplementedSessionServer returns a brand new Login service
func ImplementedSessionServer() *Service {
	return &session{}
}
