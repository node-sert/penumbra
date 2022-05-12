pub mod channel_open_init {
    use super::super::*;

    #[async_trait]
    pub trait ChannelOpenInitExecute: StateExt {
        async fn execute(&mut self, msg: &MsgChannelOpenInit) {
            let channel_id = ChannelId::new(self.get_channel_counter().await.unwrap());
            let new_channel = ChannelEnd {
                state: ChannelState::Init,
                ordering: msg.channel.ordering,
                remote: msg.channel.remote.clone(),
                connection_hops: msg.channel.connection_hops.clone(),
                version: msg.channel.version.clone(),
            };

            self.put_channel(&channel_id, &msg.port_id, new_channel)
                .await;
            self.put_send_sequence(&channel_id, &msg.port_id, 1).await;
            self.put_recv_sequence(&channel_id, &msg.port_id, 1).await;
            self.put_ack_sequence(&channel_id, &msg.port_id, 1).await;
        }
    }

    impl<T: StateExt> ChannelOpenInitExecute for T {}
}
