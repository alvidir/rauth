package session

const (
	errCookieAlreadyExists   = "Session for cookie %v already exists"
	errProviderAlreadyExists = "Provider name already exists"
	errClientNotExists       = "Session for client %v does not exists"
	errCookieNotExists       = "Session for cookie %s does not exists"
	errAssertionFailed       = "Sessions assertion has failed for cookie %v"
	errNoDeadline            = "Session requires a deadline"
)
