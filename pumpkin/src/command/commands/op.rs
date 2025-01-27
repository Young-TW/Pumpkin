use crate::{
    command::{
        args::{players::PlayersArgumentConsumer, Arg, ConsumedArgs},
        tree::CommandTree,
        tree_builder::argument,
        CommandError, CommandExecutor, CommandSender,
    },
    data::{op_data::OPERATOR_CONFIG, SaveJSONConfiguration},
};
use async_trait::async_trait;
use pumpkin_config::{op::Op, BASIC_CONFIG};
use pumpkin_util::text::TextComponent;
use CommandError::InvalidConsumption;

const NAMES: [&str; 1] = ["op"];
const DESCRIPTION: &str = "Grants operator status to a player.";
const ARG_TARGETS: &str = "targets";

struct OpExecutor;

#[async_trait]
impl CommandExecutor for OpExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let mut config = OPERATOR_CONFIG.write().await;

        let Some(Arg::Players(targets)) = args.get(&ARG_TARGETS) else {
            return Err(InvalidConsumption(Some(ARG_TARGETS.into())));
        };

        for player in targets {
            let new_level = BASIC_CONFIG
                .op_permission_level
                .min(sender.permission_lvl());

            if player.permission_lvl.load() == new_level {
                sender
                    .send_message(TextComponent::translate("commands.op.failed", [].into()))
                    .await;
                continue;
            }

            if let Some(op) = config
                .ops
                .iter_mut()
                .find(|o| o.uuid == player.gameprofile.id)
            {
                op.level = new_level;
            } else {
                let op_entry = Op::new(
                    player.gameprofile.id,
                    player.gameprofile.name.clone(),
                    new_level,
                    false,
                );
                config.ops.push(op_entry);
            }

            config.save();

            player
                .set_permission_lvl(new_level, &server.command_dispatcher)
                .await;

            let player_name = &player.gameprofile.name;
            sender
                .send_message(TextComponent::translate(
                    "commands.op.success",
                    [TextComponent::text(player_name.clone())].into(),
                ))
                .await;
        }

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION)
        .then(argument(ARG_TARGETS, PlayersArgumentConsumer).execute(OpExecutor))
}