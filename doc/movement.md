# Movement Handling

The high-level procedure for movement handling is:

 1. Player presses some inputs.
 2. Client runs physics from inputs.  Results are displayed to player and also
    sent to the server.
 3. Server also runs physics from inputs, and checks that the client's results
    are valid.

Sending the predicted motion from the client to the server allows the server to
detect mispredicts and take corrective action immediately.


## Normal client behavior

The client receives a stream of input events from the keyboard.  It runs
physics based on those events to compute the player character's expected
motion.

The client always renders the expected motion directly, with no separate
prediction or "future" time delay.  Essentially, the client treats its own
physics computations as if they were authoritative.  The only exception is in
the event of a conflict - see below.

The client sends all input events and all changes to the expected motion to the
server.  First, whenever the current input changes from zero to non-zero, the
client sends a PathStart message to establish a baseline time.  Then, the
client sends PathUpdate and PathBlocked messages (corresponding to motion start
and motion end events) with timestamps given relative to that baseline.
PathUpdate also contains the current input (the input is never sent alone).

The PathStart message contains what the client believes to be the player
character's current position.  This ensures the client and server are in sync
at the start of the path.  (If they aren't in sync, the server will detect a
conflict.)  Later messages do not send a position, as it can be computed based
on the start position and the previous messages.


## Normal server behavior

The server keeps track of the current input based on input updates applied so
far, and also keeps a queue of pending motion updates.  On each tick, it runs
physics based on the current input, and checks changes to the character's
actual motion against the expected motion updates reported by the client.  If
they match, everything proceeds normally.  If there is a mismatch, the server
has detected a conflict - see below.

On receiving a PathStart message, the server should immediately update the base
timestamp for the character, as messages with relative timestamps may arrive
before the PathStart has been processed.  The new base timestamp should be set
a short time in the future, to account for jitter in packet travel times, but
it must not be set to a time earlier than the last event currently in the
queue.

At the start of each tick, any events whose timestamps match the current time
will be applied.  For PathStart, the server checks that the character's
position matches the position in the PathStart message (otherwise, there is a
conflict).  For PathUpdate, it updates the current input and expected motion.
For PathBlocked, it updates the expected motion only.

Next, the server runs physics based on the current input.  If the motion
afterward matches the character's expected motion, then everything proceeds
normally.  Otherwise, there is a conflict.


## Conflict handling

A conflict arises when the server detects that the character is about to move
in a way that the client did not expect (based on the updates received from the
client so far).  The server resolves conflicts by stopping any current motion
and notifying the client with a ResetMotion message.

On the server side, the server handles a conflict by setting the character's
current motion to "stationary" and deleting the entire queue.  The character
begins moving again only once the client has sent a new PathStart with an
up-to-date position.  Any messages sent prior to that PathStart will be
ignored.

On the client side, when the client receives a ResetMotion message, it replaces
the expected motion with the motion given in the message.  This causes the
character to snap back to its actual position according to the server.  Then,
if the player is pressing some input keys, the client will send PathStart and
PathUpdate messages as if the player had just pressed those keys.  So from the
player's perspective, the character will snap back but continue moving in the
direction they are inputting.  (From other players' perspectives, the character
will stop at the position where the conflict arose, then begin moving again
after a short delay.)


## Interaction with activities and other server-controlled motion updates

Whenever the character is engaged in an uninterruptible activity, the client
behaves as if the player is inputting no motion.  When the activity ends, it
will send a PathStart and proceed as normal.  This means the player will
observe no delay between finishing the activity and moving, but other players
will see a short delay based on the controlling player's ping.

When the player is inputting no motion (or during an uninterruptible activity),
the client will overwrite the expected motion with any motion it receives from
the server.  This means emotes will work as expected when standing still.  (If
not standing still, the motion change to begin the emote will be accompanied by
a ResetMotion, and then the client will send its inputs, causing the character
to cancel the emote and begin moving again.) Uninterruptible activities will
also work, though the activity change should be sent before the motion change
to avoid a redundant ResetMotion message.
