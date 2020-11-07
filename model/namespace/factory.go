package namespace

// New builds a brand new namespace
func New() *Namespace {
	return &Namespace{
		Entries: make(map[string]entry),
	}
}
