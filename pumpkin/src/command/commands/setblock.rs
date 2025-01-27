use async_trait::async_trait;
use pumpkin_util::text::TextComponent;

use crate::command::args::block::BlockArgumentConsumer;
use crate::command::args::position_block::BlockPosArgumentConsumer;
use crate::command::args::{ConsumedArgs, FindArg};
use crate::command::tree::CommandTree;
use crate::command::tree_builder::{argument, literal};
use crate::command::{CommandError, CommandExecutor, CommandSender};

const NAMES: [&str; 1] = ["setblock"];

const DESCRIPTION: &str = "Place a block.";

const ARG_BLOCK: &str = "block";
const ARG_BLOCK_POS: &str = "pos";

#[derive(Clone, Copy)]
enum Mode {
    /// with particles + item drops
    Destroy,

    /// only replaces air
    Keep,

    /// default; without particles
    Replace,
}

struct SetblockExecutor(Mode);

#[async_trait]
impl CommandExecutor for SetblockExecutor {
    async fn execute<'a>(
        &self,
        sender: &mut CommandSender<'a>,
        _server: &crate::server::Server,
        args: &ConsumedArgs<'a>,
    ) -> Result<(), CommandError> {
        let block = BlockArgumentConsumer::find_arg(args, ARG_BLOCK)?;
        let block_state_id = block.default_state_id;
        let pos = BlockPosArgumentConsumer::find_arg(args, ARG_BLOCK_POS)?;
        let mode = self.0;
        // TODO: allow console to use the command (seed sender.world)
        let world = sender.world().ok_or(CommandError::InvalidRequirement)?;

        let success = match mode {
            Mode::Destroy => {
                world.break_block(&pos, None).await;
                world.set_block_state(&pos, block_state_id).await;
                true
            }
            Mode::Replace => {
                world.set_block_state(&pos, block_state_id).await;
                true
            }
            Mode::Keep => match world.get_block_state(&pos).await {
                Ok(old_state) if old_state.air => {
                    world.set_block_state(&pos, block_state_id).await;
                    true
                }
                Ok(_) => false,
                Err(e) => return Err(CommandError::OtherPumpkin(e.into())),
            },
        };

        sender
            .send_message(if success {
                TextComponent::translate(
                    "commands.setblock.success",
                    [
                        TextComponent::text(pos.0.x.to_string()),
                        TextComponent::text(pos.0.y.to_string()),
                        TextComponent::text(pos.0.z.to_string()),
                    ]
                    .into(),
                )
            } else {
                TextComponent::translate("commands.setblock.failed", [].into())
            })
            .await;

        Ok(())
    }
}

pub fn init_command_tree() -> CommandTree {
    CommandTree::new(NAMES, DESCRIPTION).then(
        argument(ARG_BLOCK_POS, BlockPosArgumentConsumer).then(
            argument(ARG_BLOCK, BlockArgumentConsumer)
                .then(literal("replace").execute(SetblockExecutor(Mode::Replace)))
                .then(literal("destroy").execute(SetblockExecutor(Mode::Destroy)))
                .then(literal("keep").execute(SetblockExecutor(Mode::Keep)))
                .execute(SetblockExecutor(Mode::Replace)),
        ),
    )
}