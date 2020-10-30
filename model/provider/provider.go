package provider

type provider struct {
	name string
}

func (provider *provider) Name() string {
	return provider.name
}
