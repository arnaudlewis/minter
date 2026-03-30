spec design-chat v1.0.0
title "Design Chat Panel"

description
  An integrated chat panel in the Design view that connects to the
  user's Claude Code CLI for real-time design iteration. When the
  server detects the claude CLI in PATH, the Design view shows a
  split layout with the chat panel on the left and the design
  preview on the right. The user types natural language design
  requests in the chat, Claude responds with text and calls MCP
  design tools, and the preview updates automatically. The chat
  is entirely optional — the design system works without it. Users
  without Claude Code see the full-width preview with guidance on
  how to install it.

motivation
  Design iteration through MCP tool calls alone requires the user
  to switch between their editor and the preview. An integrated
  chat panel keeps the conversation and the visual result side by
  side, reducing context switches. Streaming responses let the user
  see progress immediately rather than waiting for a full response.
  Making the chat optional ensures the core design workflow is never
  blocked by a missing dependency.

nfr
  operability#zero-config
  reliability#no-silent-data-loss


# Detection

behavior detect-claude-available [happy_path]
  "Server detects the claude CLI and reports it as available"

  given
    The claude CLI is installed and present in PATH

  when the server starts

  then
    assert server state includes has_claude == true


behavior detect-claude-unavailable [happy_path]
  "Server reports claude as unavailable when the CLI is not found"

  given
    The claude CLI is not present in PATH

  when the server starts

  then
    assert server state includes has_claude == false


# UI layout

behavior chat-panel-visible-when-claude-available [happy_path]
  "Design view shows a split layout with chat on the left and preview on the right"

  given
    Server state has has_claude == true

  when the user navigates to the Design view

  then
    assert the chat panel is visible on the left side
    assert the design preview is visible on the right side
    assert both panels are usable simultaneously


behavior chat-panel-hidden-when-no-claude [happy_path]
  "Design view shows full-width preview with install guidance when claude is not available"

  given
    Server state has has_claude == false

  when the user navigates to the Design view

  then
    assert the design preview occupies the full width
    assert a message explains how to install Claude Code for chat-based iteration
    assert the chat panel is not visible


behavior chat-panel-collapsible [happy_path]
  "User can collapse the chat panel to give full width to the preview"

  given
    The Design view is showing the split layout with chat panel visible

  when the user collapses the chat panel

  then
    assert the design preview expands to full width
    assert the chat panel can be re-expanded
    assert the chat session is not interrupted by collapsing


behavior chat-panel-only-in-design-view [edge_case]
  "The chat panel is not visible outside the Design view"

  given
    The user is viewing the Specs or NFRs section of the dashboard

  when the dashboard renders

  then
    assert no chat panel is visible
    assert the standard dashboard layout is used


# Chat session

behavior start-chat-session [happy_path]
  "Entering the Design view starts a chat session with Claude"

  given
    Server state has has_claude == true
    No active chat session exists

  when the user navigates to the Design view

  then
    assert a chat session is initialized
    assert the chat panel shows an empty conversation
    assert the chat input is ready to accept messages


behavior send-message [happy_path]
  "User sends a message through the chat and it is delivered to Claude"

  given
    A chat session is active in the Design view

  when the user types a message and sends it

  then
    assert the message appears in the chat conversation
    assert the message is delivered to Claude for processing


behavior receive-response [happy_path]
  "Claude's text response appears in the chat panel"

  given
    A chat session is active
    The user has sent a message

  when Claude generates a response

  then
    assert the response text appears in the chat conversation
    assert the response is attributed to Claude


behavior streaming-response [happy_path]
  "Responses appear incrementally as Claude generates them"

  given
    A chat session is active
    The user has sent a message

  when Claude begins generating a response

  then
    assert text appears token-by-token in the chat panel
    assert the user can read partial output before the response is complete


# Design integration

behavior tool-calls-update-preview [happy_path]
  "Design preview updates automatically when Claude calls a design tool"

  given
    A chat session is active
    The design preview is visible

  when Claude calls a design tool that modifies the design state

  then
    assert the design preview refreshes with the updated design
    assert no manual refresh is required
    assert the chat continues normally after the tool call


behavior initial-generate-on-first-message [happy_path]
  "First chat message triggers design generation when no design exists yet"

  given
    A chat session is active
    No design state exists

  when the user sends the first message

  then
    assert a design is generated before addressing the user's request
    assert the generated design appears in the preview
    assert the chat response reflects both the generation and the user's request


# Session lifecycle

behavior session-persists-during-view [happy_path]
  "Chat session survives navigation away from and back to the Design view"

  given
    A chat session is active with message history

  when the user navigates to Specs view and then back to Design view

  then
    assert the chat conversation history is preserved
    assert the chat session is still active
    assert the user can continue the conversation


behavior session-ends-on-close [happy_path]
  "Closing the browser terminates the chat session"

  given
    A chat session is active

  when the browser tab is closed

  then
    assert the chat session is terminated
    assert server resources for the session are released


behavior session-reconnect [error_case]
  "User can reconnect when the chat session is lost unexpectedly"

  given
    A chat session was active

  when the chat session is lost due to an unexpected failure

  then
    assert the chat panel shows an error message
    assert a reconnect action is available
    assert clicking reconnect starts a new chat session


# Error cases

behavior claude-not-authenticated [error_case]
  "Show authentication guidance when the claude CLI is not logged in"

  given
    The claude CLI is present in PATH
    The claude CLI is not authenticated

  when a chat session attempts to start

  then
    assert the chat panel shows an authentication error
    assert the error includes guidance on how to authenticate
    assert the design preview remains usable without chat


behavior claude-process-crash [error_case]
  "Recover gracefully when the chat session fails mid-conversation"

  given
    A chat session is active with message history

  when the underlying chat process crashes

  then
    assert the chat panel shows an error message
    assert a reconnect action is available
    assert the design preview is not affected by the crash


behavior mcp-tool-error [error_case]
  "Tool call errors appear naturally in the chat conversation"

  given
    A chat session is active
    Claude attempts a design tool call

  when the tool call fails with an error

  then
    assert the error information appears in the chat as part of Claude's response
    assert the chat session remains active
    assert the user can continue the conversation


# Edge cases

behavior long-running-generation [edge_case]
  "Show a typing indicator while design generation is in progress"

  given
    A chat session is active
    Claude has called a design tool

  when the tool call takes multiple seconds to complete

  then
    assert a typing indicator is visible in the chat panel
    assert the indicator disappears when the tool call completes
    assert the chat is not blocked from displaying other updates


behavior multiple-tabs [edge_case]
  "Multiple browser tabs share a single chat session"

  given
    A chat session is active in one browser tab

  when a second browser tab opens the Design view

  then
    assert both tabs show the same chat conversation
    assert messages sent from either tab appear in both
    assert only one chat session exists on the server


depends on design-preview >= 1.1.0
depends on web-command >= 1.1.0
