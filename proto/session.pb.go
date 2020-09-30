// Code generated by protoc-gen-go. DO NOT EDIT.
// versions:
// 	protoc-gen-go v1.23.0
// 	protoc        v3.6.1
// source: proto/session/session.proto

package session

import (
	context "context"
	proto "github.com/golang/protobuf/proto"
	grpc "google.golang.org/grpc"
	codes "google.golang.org/grpc/codes"
	status "google.golang.org/grpc/status"
	protoreflect "google.golang.org/protobuf/reflect/protoreflect"
	protoimpl "google.golang.org/protobuf/runtime/protoimpl"
	reflect "reflect"
	sync "sync"
)

const (
	// Verify that this generated code is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(20 - protoimpl.MinVersion)
	// Verify that runtime/protoimpl is sufficiently up-to-date.
	_ = protoimpl.EnforceVersion(protoimpl.MaxVersion - 20)
)

// This is a compile-time assertion that a sufficiently up-to-date version
// of the legacy proto package is being used.
const _ = proto.ProtoPackageIsVersion4

type Status int32

const (
	Status_ALIVE Status = 0
	Status_OLD   Status = 1
	Status_DEAD  Status = 2
	Status_NEW   Status = 3
)

// Enum value maps for Status.
var (
	Status_name = map[int32]string{
		0: "ALIVE",
		1: "OLD",
		2: "DEAD",
		3: "NEW",
	}
	Status_value = map[string]int32{
		"ALIVE": 0,
		"OLD":   1,
		"DEAD":  2,
		"NEW":   3,
	}
)

func (x Status) Enum() *Status {
	p := new(Status)
	*p = x
	return p
}

func (x Status) String() string {
	return protoimpl.X.EnumStringOf(x.Descriptor(), protoreflect.EnumNumber(x))
}

func (Status) Descriptor() protoreflect.EnumDescriptor {
	return file_proto_session_session_proto_enumTypes[0].Descriptor()
}

func (Status) Type() protoreflect.EnumType {
	return &file_proto_session_session_proto_enumTypes[0]
}

func (x Status) Number() protoreflect.EnumNumber {
	return protoreflect.EnumNumber(x)
}

// Deprecated: Use Status.Descriptor instead.
func (Status) EnumDescriptor() ([]byte, []int) {
	return file_proto_session_session_proto_rawDescGZIP(), []int{0}
}

type SessionResponse struct {
	state         protoimpl.MessageState
	sizeCache     protoimpl.SizeCache
	unknownFields protoimpl.UnknownFields

	Key      string `protobuf:"bytes,1,opt,name=key,proto3" json:"key,omitempty"`                            // Session coockie
	Deadline int64  `protobuf:"varint,2,opt,name=deadline,proto3" json:"deadline,omitempty"`                 // Coockie deadline
	Status   Status `protobuf:"varint,3,opt,name=status,proto3,enum=session.Status" json:"status,omitempty"` // Session status for the given coockie
}

func (x *SessionResponse) Reset() {
	*x = SessionResponse{}
	if protoimpl.UnsafeEnabled {
		mi := &file_proto_session_session_proto_msgTypes[0]
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		ms.StoreMessageInfo(mi)
	}
}

func (x *SessionResponse) String() string {
	return protoimpl.X.MessageStringOf(x)
}

func (*SessionResponse) ProtoMessage() {}

func (x *SessionResponse) ProtoReflect() protoreflect.Message {
	mi := &file_proto_session_session_proto_msgTypes[0]
	if protoimpl.UnsafeEnabled && x != nil {
		ms := protoimpl.X.MessageStateOf(protoimpl.Pointer(x))
		if ms.LoadMessageInfo() == nil {
			ms.StoreMessageInfo(mi)
		}
		return ms
	}
	return mi.MessageOf(x)
}

// Deprecated: Use SessionResponse.ProtoReflect.Descriptor instead.
func (*SessionResponse) Descriptor() ([]byte, []int) {
	return file_proto_session_session_proto_rawDescGZIP(), []int{0}
}

func (x *SessionResponse) GetKey() string {
	if x != nil {
		return x.Key
	}
	return ""
}

func (x *SessionResponse) GetDeadline() int64 {
	if x != nil {
		return x.Deadline
	}
	return 0
}

func (x *SessionResponse) GetStatus() Status {
	if x != nil {
		return x.Status
	}
	return Status_ALIVE
}

var File_proto_session_session_proto protoreflect.FileDescriptor

var file_proto_session_session_proto_rawDesc = []byte{
	0x0a, 0x1b, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x2f, 0x73, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x2f,
	0x73, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x12, 0x07, 0x73,
	0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x1a, 0x19, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x2f, 0x73, 0x65,
	0x73, 0x73, 0x69, 0x6f, 0x6e, 0x2f, 0x6c, 0x6f, 0x67, 0x69, 0x6e, 0x2e, 0x70, 0x72, 0x6f, 0x74,
	0x6f, 0x1a, 0x1a, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x2f, 0x73, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e,
	0x2f, 0x6c, 0x6f, 0x67, 0x6f, 0x75, 0x74, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x1a, 0x1a, 0x70,
	0x72, 0x6f, 0x74, 0x6f, 0x2f, 0x73, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x2f, 0x73, 0x69, 0x67,
	0x6e, 0x75, 0x70, 0x2e, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x22, 0x68, 0x0a, 0x0f, 0x53, 0x65, 0x73,
	0x73, 0x69, 0x6f, 0x6e, 0x52, 0x65, 0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65, 0x12, 0x10, 0x0a, 0x03,
	0x6b, 0x65, 0x79, 0x18, 0x01, 0x20, 0x01, 0x28, 0x09, 0x52, 0x03, 0x6b, 0x65, 0x79, 0x12, 0x1a,
	0x0a, 0x08, 0x64, 0x65, 0x61, 0x64, 0x6c, 0x69, 0x6e, 0x65, 0x18, 0x02, 0x20, 0x01, 0x28, 0x03,
	0x52, 0x08, 0x64, 0x65, 0x61, 0x64, 0x6c, 0x69, 0x6e, 0x65, 0x12, 0x27, 0x0a, 0x06, 0x73, 0x74,
	0x61, 0x74, 0x75, 0x73, 0x18, 0x03, 0x20, 0x01, 0x28, 0x0e, 0x32, 0x0f, 0x2e, 0x73, 0x65, 0x73,
	0x73, 0x69, 0x6f, 0x6e, 0x2e, 0x53, 0x74, 0x61, 0x74, 0x75, 0x73, 0x52, 0x06, 0x73, 0x74, 0x61,
	0x74, 0x75, 0x73, 0x2a, 0x2f, 0x0a, 0x06, 0x53, 0x74, 0x61, 0x74, 0x75, 0x73, 0x12, 0x09, 0x0a,
	0x05, 0x41, 0x4c, 0x49, 0x56, 0x45, 0x10, 0x00, 0x12, 0x07, 0x0a, 0x03, 0x4f, 0x4c, 0x44, 0x10,
	0x01, 0x12, 0x08, 0x0a, 0x04, 0x44, 0x45, 0x41, 0x44, 0x10, 0x02, 0x12, 0x07, 0x0a, 0x03, 0x4e,
	0x45, 0x57, 0x10, 0x03, 0x32, 0x81, 0x02, 0x0a, 0x07, 0x53, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e,
	0x12, 0x38, 0x0a, 0x05, 0x4c, 0x6f, 0x67, 0x69, 0x6e, 0x12, 0x15, 0x2e, 0x73, 0x65, 0x73, 0x73,
	0x69, 0x6f, 0x6e, 0x2e, 0x4c, 0x6f, 0x67, 0x69, 0x6e, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74,
	0x1a, 0x18, 0x2e, 0x73, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x2e, 0x53, 0x65, 0x73, 0x73, 0x69,
	0x6f, 0x6e, 0x52, 0x65, 0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65, 0x12, 0x44, 0x0a, 0x0b, 0x47, 0x6f,
	0x6f, 0x67, 0x6c, 0x65, 0x4c, 0x6f, 0x67, 0x69, 0x6e, 0x12, 0x1b, 0x2e, 0x73, 0x65, 0x73, 0x73,
	0x69, 0x6f, 0x6e, 0x2e, 0x47, 0x6f, 0x6f, 0x67, 0x6c, 0x65, 0x4c, 0x6f, 0x67, 0x69, 0x6e, 0x52,
	0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x1a, 0x18, 0x2e, 0x73, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e,
	0x2e, 0x53, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x52, 0x65, 0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65,
	0x12, 0x3a, 0x0a, 0x06, 0x4c, 0x6f, 0x67, 0x6f, 0x75, 0x74, 0x12, 0x16, 0x2e, 0x73, 0x65, 0x73,
	0x73, 0x69, 0x6f, 0x6e, 0x2e, 0x4c, 0x6f, 0x67, 0x6f, 0x75, 0x74, 0x52, 0x65, 0x71, 0x75, 0x65,
	0x73, 0x74, 0x1a, 0x18, 0x2e, 0x73, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x2e, 0x53, 0x65, 0x73,
	0x73, 0x69, 0x6f, 0x6e, 0x52, 0x65, 0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65, 0x12, 0x3a, 0x0a, 0x06,
	0x53, 0x69, 0x67, 0x6e, 0x75, 0x70, 0x12, 0x16, 0x2e, 0x73, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e,
	0x2e, 0x53, 0x69, 0x67, 0x6e, 0x75, 0x70, 0x52, 0x65, 0x71, 0x75, 0x65, 0x73, 0x74, 0x1a, 0x18,
	0x2e, 0x73, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x2e, 0x53, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e,
	0x52, 0x65, 0x73, 0x70, 0x6f, 0x6e, 0x73, 0x65, 0x42, 0x2d, 0x5a, 0x2b, 0x67, 0x69, 0x74, 0x68,
	0x75, 0x62, 0x2e, 0x63, 0x6f, 0x6d, 0x2f, 0x61, 0x6c, 0x76, 0x69, 0x64, 0x69, 0x72, 0x2f, 0x6d,
	0x61, 0x73, 0x74, 0x65, 0x72, 0x6d, 0x69, 0x6e, 0x64, 0x2f, 0x70, 0x72, 0x6f, 0x74, 0x6f, 0x2f,
	0x73, 0x65, 0x73, 0x73, 0x69, 0x6f, 0x6e, 0x50, 0x00, 0x50, 0x01, 0x50, 0x02, 0x62, 0x06, 0x70,
	0x72, 0x6f, 0x74, 0x6f, 0x33,
}

var (
	file_proto_session_session_proto_rawDescOnce sync.Once
	file_proto_session_session_proto_rawDescData = file_proto_session_session_proto_rawDesc
)

func file_proto_session_session_proto_rawDescGZIP() []byte {
	file_proto_session_session_proto_rawDescOnce.Do(func() {
		file_proto_session_session_proto_rawDescData = protoimpl.X.CompressGZIP(file_proto_session_session_proto_rawDescData)
	})
	return file_proto_session_session_proto_rawDescData
}

var file_proto_session_session_proto_enumTypes = make([]protoimpl.EnumInfo, 1)
var file_proto_session_session_proto_msgTypes = make([]protoimpl.MessageInfo, 1)
var file_proto_session_session_proto_goTypes = []interface{}{
	(Status)(0),                // 0: session.Status
	(*SessionResponse)(nil),    // 1: session.SessionResponse
	(*LoginRequest)(nil),       // 2: session.LoginRequest
	(*GoogleLoginRequest)(nil), // 3: session.GoogleLoginRequest
	(*LogoutRequest)(nil),      // 4: session.LogoutRequest
	(*SignupRequest)(nil),      // 5: session.SignupRequest
}
var file_proto_session_session_proto_depIdxs = []int32{
	0, // 0: session.SessionResponse.status:type_name -> session.Status
	2, // 1: session.Session.Login:input_type -> session.LoginRequest
	3, // 2: session.Session.GoogleLogin:input_type -> session.GoogleLoginRequest
	4, // 3: session.Session.Logout:input_type -> session.LogoutRequest
	5, // 4: session.Session.Signup:input_type -> session.SignupRequest
	1, // 5: session.Session.Login:output_type -> session.SessionResponse
	1, // 6: session.Session.GoogleLogin:output_type -> session.SessionResponse
	1, // 7: session.Session.Logout:output_type -> session.SessionResponse
	1, // 8: session.Session.Signup:output_type -> session.SessionResponse
	5, // [5:9] is the sub-list for method output_type
	1, // [1:5] is the sub-list for method input_type
	1, // [1:1] is the sub-list for extension type_name
	1, // [1:1] is the sub-list for extension extendee
	0, // [0:1] is the sub-list for field type_name
}

func init() { file_proto_session_session_proto_init() }
func file_proto_session_session_proto_init() {
	if File_proto_session_session_proto != nil {
		return
	}
	file_proto_session_login_proto_init()
	file_proto_session_logout_proto_init()
	file_proto_session_signup_proto_init()
	if !protoimpl.UnsafeEnabled {
		file_proto_session_session_proto_msgTypes[0].Exporter = func(v interface{}, i int) interface{} {
			switch v := v.(*SessionResponse); i {
			case 0:
				return &v.state
			case 1:
				return &v.sizeCache
			case 2:
				return &v.unknownFields
			default:
				return nil
			}
		}
	}
	type x struct{}
	out := protoimpl.TypeBuilder{
		File: protoimpl.DescBuilder{
			GoPackagePath: reflect.TypeOf(x{}).PkgPath(),
			RawDescriptor: file_proto_session_session_proto_rawDesc,
			NumEnums:      1,
			NumMessages:   1,
			NumExtensions: 0,
			NumServices:   1,
		},
		GoTypes:           file_proto_session_session_proto_goTypes,
		DependencyIndexes: file_proto_session_session_proto_depIdxs,
		EnumInfos:         file_proto_session_session_proto_enumTypes,
		MessageInfos:      file_proto_session_session_proto_msgTypes,
	}.Build()
	File_proto_session_session_proto = out.File
	file_proto_session_session_proto_rawDesc = nil
	file_proto_session_session_proto_goTypes = nil
	file_proto_session_session_proto_depIdxs = nil
}

// Reference imports to suppress errors if they are not otherwise used.
var _ context.Context
var _ grpc.ClientConnInterface

// This is a compile-time assertion to ensure that this generated file
// is compatible with the grpc package it is being compiled against.
const _ = grpc.SupportPackageIsVersion6

// SessionClient is the client API for Session service.
//
// For semantics around ctx use and closing/ending streaming RPCs, please refer to https://godoc.org/google.golang.org/grpc#ClientConn.NewStream.
type SessionClient interface {
	Login(ctx context.Context, in *LoginRequest, opts ...grpc.CallOption) (*SessionResponse, error)
	GoogleLogin(ctx context.Context, in *GoogleLoginRequest, opts ...grpc.CallOption) (*SessionResponse, error)
	Logout(ctx context.Context, in *LogoutRequest, opts ...grpc.CallOption) (*SessionResponse, error)
	Signup(ctx context.Context, in *SignupRequest, opts ...grpc.CallOption) (*SessionResponse, error)
}

type sessionClient struct {
	cc grpc.ClientConnInterface
}

func NewSessionClient(cc grpc.ClientConnInterface) SessionClient {
	return &sessionClient{cc}
}

func (c *sessionClient) Login(ctx context.Context, in *LoginRequest, opts ...grpc.CallOption) (*SessionResponse, error) {
	out := new(SessionResponse)
	err := c.cc.Invoke(ctx, "/session.Session/Login", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *sessionClient) GoogleLogin(ctx context.Context, in *GoogleLoginRequest, opts ...grpc.CallOption) (*SessionResponse, error) {
	out := new(SessionResponse)
	err := c.cc.Invoke(ctx, "/session.Session/GoogleLogin", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *sessionClient) Logout(ctx context.Context, in *LogoutRequest, opts ...grpc.CallOption) (*SessionResponse, error) {
	out := new(SessionResponse)
	err := c.cc.Invoke(ctx, "/session.Session/Logout", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

func (c *sessionClient) Signup(ctx context.Context, in *SignupRequest, opts ...grpc.CallOption) (*SessionResponse, error) {
	out := new(SessionResponse)
	err := c.cc.Invoke(ctx, "/session.Session/Signup", in, out, opts...)
	if err != nil {
		return nil, err
	}
	return out, nil
}

// SessionServer is the server API for Session service.
type SessionServer interface {
	Login(context.Context, *LoginRequest) (*SessionResponse, error)
	GoogleLogin(context.Context, *GoogleLoginRequest) (*SessionResponse, error)
	Logout(context.Context, *LogoutRequest) (*SessionResponse, error)
	Signup(context.Context, *SignupRequest) (*SessionResponse, error)
}

// UnimplementedSessionServer can be embedded to have forward compatible implementations.
type UnimplementedSessionServer struct {
}

func (*UnimplementedSessionServer) Login(context.Context, *LoginRequest) (*SessionResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method Login not implemented")
}
func (*UnimplementedSessionServer) GoogleLogin(context.Context, *GoogleLoginRequest) (*SessionResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method GoogleLogin not implemented")
}
func (*UnimplementedSessionServer) Logout(context.Context, *LogoutRequest) (*SessionResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method Logout not implemented")
}
func (*UnimplementedSessionServer) Signup(context.Context, *SignupRequest) (*SessionResponse, error) {
	return nil, status.Errorf(codes.Unimplemented, "method Signup not implemented")
}

func RegisterSessionServer(s *grpc.Server, srv SessionServer) {
	s.RegisterService(&_Session_serviceDesc, srv)
}

func _Session_Login_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(LoginRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(SessionServer).Login(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/session.Session/Login",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(SessionServer).Login(ctx, req.(*LoginRequest))
	}
	return interceptor(ctx, in, info, handler)
}

func _Session_GoogleLogin_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(GoogleLoginRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(SessionServer).GoogleLogin(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/session.Session/GoogleLogin",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(SessionServer).GoogleLogin(ctx, req.(*GoogleLoginRequest))
	}
	return interceptor(ctx, in, info, handler)
}

func _Session_Logout_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(LogoutRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(SessionServer).Logout(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/session.Session/Logout",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(SessionServer).Logout(ctx, req.(*LogoutRequest))
	}
	return interceptor(ctx, in, info, handler)
}

func _Session_Signup_Handler(srv interface{}, ctx context.Context, dec func(interface{}) error, interceptor grpc.UnaryServerInterceptor) (interface{}, error) {
	in := new(SignupRequest)
	if err := dec(in); err != nil {
		return nil, err
	}
	if interceptor == nil {
		return srv.(SessionServer).Signup(ctx, in)
	}
	info := &grpc.UnaryServerInfo{
		Server:     srv,
		FullMethod: "/session.Session/Signup",
	}
	handler := func(ctx context.Context, req interface{}) (interface{}, error) {
		return srv.(SessionServer).Signup(ctx, req.(*SignupRequest))
	}
	return interceptor(ctx, in, info, handler)
}

var _Session_serviceDesc = grpc.ServiceDesc{
	ServiceName: "session.Session",
	HandlerType: (*SessionServer)(nil),
	Methods: []grpc.MethodDesc{
		{
			MethodName: "Login",
			Handler:    _Session_Login_Handler,
		},
		{
			MethodName: "GoogleLogin",
			Handler:    _Session_GoogleLogin_Handler,
		},
		{
			MethodName: "Logout",
			Handler:    _Session_Logout_Handler,
		},
		{
			MethodName: "Signup",
			Handler:    _Session_Signup_Handler,
		},
	},
	Streams:  []grpc.StreamDesc{},
	Metadata: "proto/session/session.proto",
}
