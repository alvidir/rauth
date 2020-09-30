package service

import "google.golang.org/grpc"

// Service is the trait any grpc service has to implement
type Service interface {
	RegisterServer(*grpc.Server)
}
