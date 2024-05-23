/// Generated client implementations.
#[cfg(feature = "rpc")]
pub mod query_client {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    use tonic::codegen::http::Uri;
    /// Query provides defines the gRPC querier service.
    #[derive(Debug, Clone)]
    pub struct QueryClient<T> {
        inner: tonic::client::Grpc<T>,
    }
    impl QueryClient<tonic::transport::Channel> {
        /// Attempt to create a new client by connecting to a given endpoint.
        pub async fn connect<D>(dst: D) -> Result<Self, tonic::transport::Error>
        where
            D: TryInto<tonic::transport::Endpoint>,
            D::Error: Into<StdError>,
        {
            let conn = tonic::transport::Endpoint::new(dst)?.connect().await?;
            Ok(Self::new(conn))
        }
    }
    impl<T> QueryClient<T>
    where
        T: tonic::client::GrpcService<tonic::body::BoxBody>,
        T::Error: Into<StdError>,
        T::ResponseBody: Body<Data = Bytes> + Send + 'static,
        <T::ResponseBody as Body>::Error: Into<StdError> + Send,
    {
        pub fn new(inner: T) -> Self {
            let inner = tonic::client::Grpc::new(inner);
            Self { inner }
        }
        pub fn with_origin(inner: T, origin: Uri) -> Self {
            let inner = tonic::client::Grpc::with_origin(inner, origin);
            Self { inner }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> QueryClient<InterceptedService<T, F>>
        where
            F: tonic::service::Interceptor,
            T::ResponseBody: Default,
            T: tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
                Response = http::Response<
                    <T as tonic::client::GrpcService<tonic::body::BoxBody>>::ResponseBody,
                >,
            >,
            <T as tonic::codegen::Service<
                http::Request<tonic::body::BoxBody>,
            >>::Error: Into<StdError> + Send + Sync,
        {
            QueryClient::new(InterceptedService::new(inner, interceptor))
        }
        /// Compress requests with the given encoding.
        ///
        /// This requires the server to support it otherwise it might respond with an
        /// error.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.send_compressed(encoding);
            self
        }
        /// Enable decompressing responses.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.inner = self.inner.accept_compressed(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_decoding_message_size(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.inner = self.inner.max_encoding_message_size(limit);
            self
        }
        /// DenomTrace queries a denomination trace information.
        pub async fn denom_trace(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::applications::transfer::v1::QueryDenomTraceRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<
                ::ibc_proto::ibc::applications::transfer::v1::QueryDenomTraceResponse,
            >,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.transfer.v1.Query/DenomTrace",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.applications.transfer.v1.Query", "DenomTrace"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// DenomTraces queries all denomination traces.
        pub async fn denom_traces(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::applications::transfer::v1::QueryDenomTracesRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<
                ::ibc_proto::ibc::applications::transfer::v1::QueryDenomTracesResponse,
            >,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.transfer.v1.Query/DenomTraces",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.applications.transfer.v1.Query", "DenomTraces"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// Params queries all parameters of the ibc-transfer module.
        pub async fn params(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::applications::transfer::v1::QueryParamsRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<
                ::ibc_proto::ibc::applications::transfer::v1::QueryParamsResponse,
            >,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.transfer.v1.Query/Params",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(GrpcMethod::new("ibc.applications.transfer.v1.Query", "Params"));
            self.inner.unary(req, path, codec).await
        }
        /// DenomHash queries a denomination hash information.
        pub async fn denom_hash(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::applications::transfer::v1::QueryDenomHashRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<
                ::ibc_proto::ibc::applications::transfer::v1::QueryDenomHashResponse,
            >,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.transfer.v1.Query/DenomHash",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new("ibc.applications.transfer.v1.Query", "DenomHash"),
                );
            self.inner.unary(req, path, codec).await
        }
        /// EscrowAddress returns the escrow address for a particular port and channel id.
        pub async fn escrow_address(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::applications::transfer::v1::QueryEscrowAddressRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<
                ::ibc_proto::ibc::applications::transfer::v1::QueryEscrowAddressResponse,
            >,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.transfer.v1.Query/EscrowAddress",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "ibc.applications.transfer.v1.Query",
                        "EscrowAddress",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
        /// TotalEscrowForDenom returns the total amount of tokens in escrow based on the denom.
        pub async fn total_escrow_for_denom(
            &mut self,
            request: impl tonic::IntoRequest<
                ::ibc_proto::ibc::applications::transfer::v1::QueryTotalEscrowForDenomRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<
                ::ibc_proto::ibc::applications::transfer::v1::QueryTotalEscrowForDenomResponse,
            >,
            tonic::Status,
        > {
            self.inner
                .ready()
                .await
                .map_err(|e| {
                    tonic::Status::new(
                        tonic::Code::Unknown,
                        format!("Service was not ready: {}", e.into()),
                    )
                })?;
            let codec = tonic::codec::ProstCodec::default();
            let path = http::uri::PathAndQuery::from_static(
                "/ibc.applications.transfer.v1.Query/TotalEscrowForDenom",
            );
            let mut req = request.into_request();
            req.extensions_mut()
                .insert(
                    GrpcMethod::new(
                        "ibc.applications.transfer.v1.Query",
                        "TotalEscrowForDenom",
                    ),
                );
            self.inner.unary(req, path, codec).await
        }
    }
}
/// Generated server implementations.
#[cfg(feature = "rpc")]
pub mod query_server {
    #![allow(unused_variables, dead_code, missing_docs, clippy::let_unit_value)]
    use tonic::codegen::*;
    /// Generated trait containing gRPC methods that should be implemented for use with QueryServer.
    #[async_trait]
    pub trait Query: Send + Sync + 'static {
        /// DenomTrace queries a denomination trace information.
        async fn denom_trace(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::applications::transfer::v1::QueryDenomTraceRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<
                ::ibc_proto::ibc::applications::transfer::v1::QueryDenomTraceResponse,
            >,
            tonic::Status,
        >;
        /// DenomTraces queries all denomination traces.
        async fn denom_traces(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::applications::transfer::v1::QueryDenomTracesRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<
                ::ibc_proto::ibc::applications::transfer::v1::QueryDenomTracesResponse,
            >,
            tonic::Status,
        >;
        /// Params queries all parameters of the ibc-transfer module.
        async fn params(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::applications::transfer::v1::QueryParamsRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<
                ::ibc_proto::ibc::applications::transfer::v1::QueryParamsResponse,
            >,
            tonic::Status,
        >;
        /// DenomHash queries a denomination hash information.
        async fn denom_hash(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::applications::transfer::v1::QueryDenomHashRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<
                ::ibc_proto::ibc::applications::transfer::v1::QueryDenomHashResponse,
            >,
            tonic::Status,
        >;
        /// EscrowAddress returns the escrow address for a particular port and channel id.
        async fn escrow_address(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::applications::transfer::v1::QueryEscrowAddressRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<
                ::ibc_proto::ibc::applications::transfer::v1::QueryEscrowAddressResponse,
            >,
            tonic::Status,
        >;
        /// TotalEscrowForDenom returns the total amount of tokens in escrow based on the denom.
        async fn total_escrow_for_denom(
            &self,
            request: tonic::Request<
                ::ibc_proto::ibc::applications::transfer::v1::QueryTotalEscrowForDenomRequest,
            >,
        ) -> std::result::Result<
            tonic::Response<
                ::ibc_proto::ibc::applications::transfer::v1::QueryTotalEscrowForDenomResponse,
            >,
            tonic::Status,
        >;
    }
    /// Query provides defines the gRPC querier service.
    #[derive(Debug)]
    pub struct QueryServer<T: Query> {
        inner: _Inner<T>,
        accept_compression_encodings: EnabledCompressionEncodings,
        send_compression_encodings: EnabledCompressionEncodings,
        max_decoding_message_size: Option<usize>,
        max_encoding_message_size: Option<usize>,
    }
    struct _Inner<T>(Arc<T>);
    impl<T: Query> QueryServer<T> {
        pub fn new(inner: T) -> Self {
            Self::from_arc(Arc::new(inner))
        }
        pub fn from_arc(inner: Arc<T>) -> Self {
            let inner = _Inner(inner);
            Self {
                inner,
                accept_compression_encodings: Default::default(),
                send_compression_encodings: Default::default(),
                max_decoding_message_size: None,
                max_encoding_message_size: None,
            }
        }
        pub fn with_interceptor<F>(
            inner: T,
            interceptor: F,
        ) -> InterceptedService<Self, F>
        where
            F: tonic::service::Interceptor,
        {
            InterceptedService::new(Self::new(inner), interceptor)
        }
        /// Enable decompressing requests with the given encoding.
        #[must_use]
        pub fn accept_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.accept_compression_encodings.enable(encoding);
            self
        }
        /// Compress responses with the given encoding, if the client supports it.
        #[must_use]
        pub fn send_compressed(mut self, encoding: CompressionEncoding) -> Self {
            self.send_compression_encodings.enable(encoding);
            self
        }
        /// Limits the maximum size of a decoded message.
        ///
        /// Default: `4MB`
        #[must_use]
        pub fn max_decoding_message_size(mut self, limit: usize) -> Self {
            self.max_decoding_message_size = Some(limit);
            self
        }
        /// Limits the maximum size of an encoded message.
        ///
        /// Default: `usize::MAX`
        #[must_use]
        pub fn max_encoding_message_size(mut self, limit: usize) -> Self {
            self.max_encoding_message_size = Some(limit);
            self
        }
    }
    impl<T, B> tonic::codegen::Service<http::Request<B>> for QueryServer<T>
    where
        T: Query,
        B: Body + Send + 'static,
        B::Error: Into<StdError> + Send + 'static,
    {
        type Response = http::Response<tonic::body::BoxBody>;
        type Error = std::convert::Infallible;
        type Future = BoxFuture<Self::Response, Self::Error>;
        fn poll_ready(
            &mut self,
            _cx: &mut Context<'_>,
        ) -> Poll<std::result::Result<(), Self::Error>> {
            Poll::Ready(Ok(()))
        }
        fn call(&mut self, req: http::Request<B>) -> Self::Future {
            let inner = self.inner.clone();
            match req.uri().path() {
                "/ibc.applications.transfer.v1.Query/DenomTrace" => {
                    #[allow(non_camel_case_types)]
                    struct DenomTraceSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::applications::transfer::v1::QueryDenomTraceRequest,
                    > for DenomTraceSvc<T> {
                        type Response = ::ibc_proto::ibc::applications::transfer::v1::QueryDenomTraceResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::applications::transfer::v1::QueryDenomTraceRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Query>::denom_trace(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DenomTraceSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.transfer.v1.Query/DenomTraces" => {
                    #[allow(non_camel_case_types)]
                    struct DenomTracesSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::applications::transfer::v1::QueryDenomTracesRequest,
                    > for DenomTracesSvc<T> {
                        type Response = ::ibc_proto::ibc::applications::transfer::v1::QueryDenomTracesResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::applications::transfer::v1::QueryDenomTracesRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Query>::denom_traces(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DenomTracesSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.transfer.v1.Query/Params" => {
                    #[allow(non_camel_case_types)]
                    struct ParamsSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::applications::transfer::v1::QueryParamsRequest,
                    > for ParamsSvc<T> {
                        type Response = ::ibc_proto::ibc::applications::transfer::v1::QueryParamsResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::applications::transfer::v1::QueryParamsRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Query>::params(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = ParamsSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.transfer.v1.Query/DenomHash" => {
                    #[allow(non_camel_case_types)]
                    struct DenomHashSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::applications::transfer::v1::QueryDenomHashRequest,
                    > for DenomHashSvc<T> {
                        type Response = ::ibc_proto::ibc::applications::transfer::v1::QueryDenomHashResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::applications::transfer::v1::QueryDenomHashRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Query>::denom_hash(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = DenomHashSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.transfer.v1.Query/EscrowAddress" => {
                    #[allow(non_camel_case_types)]
                    struct EscrowAddressSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::applications::transfer::v1::QueryEscrowAddressRequest,
                    > for EscrowAddressSvc<T> {
                        type Response = ::ibc_proto::ibc::applications::transfer::v1::QueryEscrowAddressResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::applications::transfer::v1::QueryEscrowAddressRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Query>::escrow_address(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = EscrowAddressSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                "/ibc.applications.transfer.v1.Query/TotalEscrowForDenom" => {
                    #[allow(non_camel_case_types)]
                    struct TotalEscrowForDenomSvc<T: Query>(pub Arc<T>);
                    impl<
                        T: Query,
                    > tonic::server::UnaryService<
                        ::ibc_proto::ibc::applications::transfer::v1::QueryTotalEscrowForDenomRequest,
                    > for TotalEscrowForDenomSvc<T> {
                        type Response = ::ibc_proto::ibc::applications::transfer::v1::QueryTotalEscrowForDenomResponse;
                        type Future = BoxFuture<
                            tonic::Response<Self::Response>,
                            tonic::Status,
                        >;
                        fn call(
                            &mut self,
                            request: tonic::Request<
                                ::ibc_proto::ibc::applications::transfer::v1::QueryTotalEscrowForDenomRequest,
                            >,
                        ) -> Self::Future {
                            let inner = Arc::clone(&self.0);
                            let fut = async move {
                                <T as Query>::total_escrow_for_denom(&inner, request).await
                            };
                            Box::pin(fut)
                        }
                    }
                    let accept_compression_encodings = self.accept_compression_encodings;
                    let send_compression_encodings = self.send_compression_encodings;
                    let max_decoding_message_size = self.max_decoding_message_size;
                    let max_encoding_message_size = self.max_encoding_message_size;
                    let inner = self.inner.clone();
                    let fut = async move {
                        let inner = inner.0;
                        let method = TotalEscrowForDenomSvc(inner);
                        let codec = tonic::codec::ProstCodec::default();
                        let mut grpc = tonic::server::Grpc::new(codec)
                            .apply_compression_config(
                                accept_compression_encodings,
                                send_compression_encodings,
                            )
                            .apply_max_message_size_config(
                                max_decoding_message_size,
                                max_encoding_message_size,
                            );
                        let res = grpc.unary(method, req).await;
                        Ok(res)
                    };
                    Box::pin(fut)
                }
                _ => {
                    Box::pin(async move {
                        Ok(
                            http::Response::builder()
                                .status(200)
                                .header("grpc-status", "12")
                                .header("content-type", "application/grpc")
                                .body(empty_body())
                                .unwrap(),
                        )
                    })
                }
            }
        }
    }
    impl<T: Query> Clone for QueryServer<T> {
        fn clone(&self) -> Self {
            let inner = self.inner.clone();
            Self {
                inner,
                accept_compression_encodings: self.accept_compression_encodings,
                send_compression_encodings: self.send_compression_encodings,
                max_decoding_message_size: self.max_decoding_message_size,
                max_encoding_message_size: self.max_encoding_message_size,
            }
        }
    }
    impl<T: Query> Clone for _Inner<T> {
        fn clone(&self) -> Self {
            Self(Arc::clone(&self.0))
        }
    }
    impl<T: std::fmt::Debug> std::fmt::Debug for _Inner<T> {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{:?}", self.0)
        }
    }
    impl<T: Query> tonic::server::NamedService for QueryServer<T> {
        const NAME: &'static str = "ibc.applications.transfer.v1.Query";
    }
}
