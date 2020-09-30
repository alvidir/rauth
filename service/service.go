package session

import (
	"context"

	pb "github.com/alvidir/session/proto"
	"google.golang.org/grpc"
)

// session represents a service for session
type session struct {
	pb.UnimplementedSessionServer
}

// RegisterServer registers the current session service into a provided grpc server
func (session *session) RegisterServer(grpcServer *grpc.Server) {
	pb.RegisterSessionServer(grpcServer, session)
}

// Login implementation for the protobuf contract
func (session *session) Login(ctx context.Context, req *pb.LoginRequest) (out *pb.SessionResponse, err error) {
	txLogin := newTxLogin()
	txLogin.Execute(ctx)

	out = &pb.SessionResponse{
		Key:      "",
		Deadline: 0,
		Status:   pb.Status_ALIVE,
	}

	return
}

// GoogleLogin implementation for the protobuf contract
func (session *session) GoogleLogin(ctx context.Context, req *pb.GoogleLoginRequest) (out *pb.SessionResponse, err error) {
	txGoogleLogin := newTxGoogleLogin()
	txGoogleLogin.Execute(ctx)

	out = &pb.SessionResponse{
		Key:      "",
		Deadline: 0,
		Status:   pb.Status_ALIVE,
	}

	return
}

// Logout implementation for the protobuf contract
func (session *session) Logout(ctx context.Context, req *pb.LogoutRequest) (out *pb.SessionResponse, err error) {
	txLogout := newTxLogout()
	txLogout.Execute(ctx)

	out = &pb.SessionResponse{
		Key:      "",
		Deadline: 0,
		Status:   pb.Status_ALIVE,
	}

	return
}

// Signup implementation for the protobuf contract
func (session *session) Signup(ctx context.Context, req *pb.SignupRequest) (out *pb.SessionResponse, err error) {
	txSignup := newTxSignup()
	txSignup.Execute(ctx)

	out = &pb.SessionResponse{
		Key:      "",
		Deadline: 0,
		Status:   pb.Status_ALIVE,
	}

	return
}
