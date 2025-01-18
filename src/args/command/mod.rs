use crate::args::command::publish::CommandPublish;
use crate::args::command::sparkplug::CommandSparkplug;
use crate::args::command::subscribe::{CommandSubscribe, OutputTarget as OutputTargetArgs};
use crate::args::ArgsError;
use clap::Subcommand;
use mqtlib::config::filter::FilterTypes;
use mqtlib::config::publish::{PublishBuilder, PublishTriggerType, PublishTriggerTypePeriodic};
use mqtlib::config::subscription::{
    Output, OutputTarget, OutputTargetConsole, OutputTargetFile, OutputTargetTopic, Subscription,
    SubscriptionBuilder,
};
use mqtlib::config::topic::{Topic, TopicBuilder};
use mqtlib::config::{PayloadType, PublishInputType, PublishInputTypeContentPath};
use mqtlib::mqtt::QoS;
use mqtlib::sparkplug::{GroupId, SPARKPLUG_TOPIC_VERSION};
use std::fmt::Display;
use std::time::Duration;

pub mod publish;
pub mod sparkplug;
pub mod subscribe;

#[derive(Clone, Debug, Subcommand)]
pub enum Command {
    #[command(name = "pub")]
    Publish(CommandPublish),
    #[command(name = "sub")]
    Subscribe(CommandSubscribe),
    #[command(name = "sparkplug", alias = "sp")]
    Sparkplug(CommandSparkplug),
}

impl Command {
    pub(crate) fn get_topics(&self) -> Result<Vec<Topic>, crate::args::ArgsError> {
        match self {
            Command::Publish(config) => Command::get_topics_for_publish(config),
            Command::Subscribe(config) => Command::get_topics_for_subscribe(config),
            Command::Sparkplug(config) => Command::get_topics_for_sparkplug(config),
        }
    }

    fn get_topics_for_publish(
        config: &CommandPublish,
    ) -> Result<Vec<Topic>, crate::args::ArgsError> {
        let mut result = Vec::new();

        let trigger = PublishTriggerType::Periodic(PublishTriggerTypePeriodic::new(
            config.interval.unwrap_or(Duration::from_secs(1)),
            config.count.or(Some(1)),
            Duration::from_millis(1000),
        ));

        let message_type = PublishInputTypeContentPath {
            content: if config.message.null_message {
                None
            } else if config.message.message.is_some() {
                Some(config.message.clone().message.unwrap().to_vec().clone())
            } else {
                None
            },
            path: if config.message.file.is_some() {
                config.message.file.clone()
            } else {
                None
            },
        };

        let message_input_type = match &config.message_type {
            None => PublishInputType::Text(message_type),
            Some(payload_type) => match payload_type {
                PublishInputType::Text(_) => PublishInputType::Text(message_type),
                PublishInputType::Raw(_) => PublishInputType::Raw(message_type.into()),
                PublishInputType::Hex(_) => PublishInputType::Hex(message_type),
                PublishInputType::Json(_) => PublishInputType::Json(message_type),
                PublishInputType::Yaml(_) => PublishInputType::Yaml(message_type),
                PublishInputType::Base64(_) => PublishInputType::Base64(message_type),
                PublishInputType::Null => {
                    PublishInputType::Text(PublishInputTypeContentPath::default())
                }
            },
        };

        let topic_type = config.topic_type.clone().unwrap_or(PayloadType::Text);

        let publish = PublishBuilder::default()
            .qos(config.qos.unwrap_or(QoS::AtLeastOnce))
            .retain(config.retain)
            .enabled(true)
            .trigger(vec![trigger])
            .input(message_input_type)
            .filters(FilterTypes::default())
            .build()?;
        let topic = TopicBuilder::default()
            .topic(config.topic.clone())
            .publish(Some(publish))
            .subscription(None)
            .payload_type(topic_type)
            .build()?;

        result.push(topic);

        Ok(result)
    }

    fn get_topics_for_subscribe(config: &CommandSubscribe) -> Result<Vec<Topic>, ArgsError> {
        let mut result = Vec::new();

        let topic_type = config.topic_type.clone().unwrap_or(PayloadType::Text);

        let output_target: OutputTarget = match &config.output_target {
            None => OutputTarget::Console(OutputTargetConsole::default()),
            Some(target) => match target {
                OutputTargetArgs::Console(_) => {
                    OutputTarget::Console(OutputTargetConsole::default())
                }
                OutputTargetArgs::File(config) => OutputTarget::File(OutputTargetFile {
                    path: config.path.clone(),
                    overwrite: config.overwrite,
                    prepend: config.prepend.clone(),
                    append: config.append.clone(),
                }),
                OutputTargetArgs::Topic(config) => OutputTarget::Topic(OutputTargetTopic {
                    topic: config.topic.clone(),
                    qos: config.qos.unwrap_or(QoS::AtLeastOnce),
                    retain: config.retain,
                }),
            },
        };

        let output = Output {
            format: config.output_type.clone().unwrap_or(PayloadType::Text),
            target: output_target,
        };

        let subscription = SubscriptionBuilder::default()
            .qos(config.qos.unwrap_or(QoS::AtLeastOnce))
            .enabled(true)
            .filters(FilterTypes::default())
            .outputs(vec![output])
            .build()?;
        let topic = TopicBuilder::default()
            .topic(config.topic.clone())
            .subscription(Some(subscription))
            .publish(None)
            .payload_type(topic_type)
            .build()?;

        result.push(topic);

        Ok(result)
    }

    fn get_topics_for_sparkplug(
        config: &CommandSparkplug,
    ) -> Result<Vec<Topic>, crate::args::ArgsError> {
        let mut result = Vec::new();

        if config.include_groups.is_empty() {
            result.append(&mut Self::add_sparkplug_topics_for_group_id(
                "+",
                config.qos.unwrap_or(QoS::AtLeastOnce),
            )?);
        } else {
            for group_id in &config.include_groups {
                result.append(&mut Self::add_sparkplug_topics_for_group_id(
                    group_id,
                    config.qos.unwrap_or(QoS::AtLeastOnce),
                )?);
            }
        }

        Ok(result)
    }

    fn add_sparkplug_topics_for_group_id<T: Into<GroupId> + Display>(
        group_id: T,
        qos: QoS,
    ) -> Result<Vec<Topic>, ArgsError> {
        fn get_subscription(qos: QoS, format: PayloadType) -> Result<Subscription, ArgsError> {
            let output = Output {
                format,
                target: OutputTarget::Console(OutputTargetConsole::default()),
            };

            Ok(SubscriptionBuilder::default()
                .qos(qos)
                .enabled(true)
                .filters(FilterTypes::default())
                .outputs(vec![output])
                .build()?)
        }
        let mut result: Vec<Topic> = vec![];

        let topic_nbirth = TopicBuilder::default()
            .topic(format!("{}/{}/NBIRTH/#", SPARKPLUG_TOPIC_VERSION, group_id))
            .subscription(Some(get_subscription(qos, PayloadType::Sparkplug)?))
            .publish(None)
            .payload_type(PayloadType::Sparkplug)
            .build()?;

        let mut topic_ndeath = topic_nbirth.clone();
        topic_ndeath.topic = format!("{}/{}/NDEATH/#", SPARKPLUG_TOPIC_VERSION, group_id);

        let mut topic_ndata = topic_nbirth.clone();
        topic_ndata.topic = format!("{}/{}/NDATA/#", SPARKPLUG_TOPIC_VERSION, group_id);

        let mut topic_ncmd = topic_nbirth.clone();
        topic_ncmd.topic = format!("{}/{}/NCMD/#", SPARKPLUG_TOPIC_VERSION, group_id);

        let mut topic_dbirth = topic_nbirth.clone();
        topic_dbirth.topic = format!("{}/{}/DBIRTH/#", SPARKPLUG_TOPIC_VERSION, group_id);

        let mut topic_ddeath = topic_ndeath.clone();
        topic_ddeath.topic = format!("{}/{}/DDEATH/#", SPARKPLUG_TOPIC_VERSION, group_id);

        let mut topic_ddata = topic_nbirth.clone();
        topic_ddata.topic = format!("{}/{}/DDATA/#", SPARKPLUG_TOPIC_VERSION, group_id);

        let mut topic_dcmd = topic_nbirth.clone();
        topic_dcmd.topic = format!("{}/{}/DCMD/#", SPARKPLUG_TOPIC_VERSION, group_id);

        let topic_state = TopicBuilder::default()
            .topic(format!("{}/{}/STATE/#", SPARKPLUG_TOPIC_VERSION, group_id))
            .subscription(Some(get_subscription(qos, PayloadType::Json)?))
            .publish(None)
            .payload_type(PayloadType::Json)
            .build()?;

        result.push(topic_nbirth);
        result.push(topic_ndata);
        result.push(topic_ndeath);
        result.push(topic_ncmd);

        result.push(topic_dbirth);
        result.push(topic_ddeath);
        result.push(topic_ddata);
        result.push(topic_dcmd);

        result.push(topic_state);

        Ok(result)
    }
}
