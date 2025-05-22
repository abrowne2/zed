use crate::agent_panel::AgentPanel;
use gpui::{prelude::*, *};
use ui::components::modal::{Modal, ModalHeader};
use ui::prelude::*;
use workspace::{CollaboratorId, ModalView};

pub struct AgentModal {
    agent_panel: Entity<AgentPanel>,
    modal_id: SharedString,
    scroll_handle: ScrollHandle,
    focus_handle: FocusHandle,
}

impl AgentModal {
    pub fn new(
        agent_panel: Entity<AgentPanel>,
        _window: &mut Window,
        cx: &mut Context<Self>,
    ) -> Self {
        let modal_id = "agent_chat_modal".into();
        let scroll_handle = ScrollHandle::new();
        let focus_handle = cx.focus_handle();

        Self {
            agent_panel,
            modal_id,
            scroll_handle,
            focus_handle,
        }
    }
}
impl Render for AgentModal {
    fn render(&mut self, window: &mut Window, modal_cx: &mut Context<Self>) -> impl IntoElement {
        let focus_handle = self.focus_handle(modal_cx);

        // First, check if we have an active thread
        let has_active_thread = self
            .agent_panel
            .update(modal_cx, |agent, cx| !agent.thread.read(cx).is_empty());

        let agent_panel = self.agent_panel.clone();

        // Create our modal content
        div()
            .elevation_3(modal_cx)
            .bottom_neg_128()
            .key_context("AgentModal")
            .track_focus(&focus_handle)
            .on_action(modal_cx.listener(|_, _: &menu::Cancel, _window, cx| {
                cx.emit(DismissEvent);
            }))
            .on_key_up(move |event: &KeyUpEvent, window, modal_cx| {
                let keystroke = event.keystroke.clone();
                let control_pressed = keystroke.modifiers.control;

                if control_pressed && keystroke.key == "e" {
                    agent_panel.update(modal_cx, |agent, cx| {
                        let workspace_ref = agent.workspace.upgrade().unwrap();

                        let following = workspace_ref.read_with(cx, |workspace, _| {
                            workspace.is_being_followed(CollaboratorId::Agent)
                        });

                        workspace_ref.update(cx, |workspace, workspace_cx| {
                            if following {
                                workspace.unfollow(CollaboratorId::Agent, window, workspace_cx);
                            } else {
                                workspace.follow(CollaboratorId::Agent, window, workspace_cx);
                            }
                        });
                    });
                } else {
                    modal_cx.propagate()
                }
            })
            .child(
                Modal::new(self.modal_id.clone(), Some(self.scroll_handle.clone()))
                    .header(
                        ModalHeader::new()
                            .headline("Agent Chat")
                            .show_dismiss_button(true),
                    )
                    // Use a simpler container for the content
                    .child(
                        v_flex()
                            .size_full()
                            .justify_between()
                            .bg(modal_cx.theme().colors().background)
                            .min_h(px(600.0))
                            .min_w(px(400.0))
                            // Directly render the thread if it exists
                            .child(if has_active_thread {
                                self.agent_panel.update(modal_cx, |agent, _cx| {
                                    agent.thread.clone().into_any_element()
                                })
                            } else {
                                // Empty state
                                v_flex()
                                    .size_full()
                                    .justify_center()
                                    .items_center()
                                    .into_any_element()
                            })
                            // Message editor at the bottom
                            .child(self.agent_panel.update(modal_cx, |agent, cx| {
                                h_flex()
                                    .child(agent.message_editor.clone())
                                    .into_any_element()
                            })),
                    ),
            )
            .into_any_element()
    }
}

impl ModalView for AgentModal {}

impl EventEmitter<DismissEvent> for AgentModal {}

impl Focusable for AgentModal {
    fn focus_handle(&self, cx: &App) -> FocusHandle {
        self.focus_handle.clone()
    }
}
