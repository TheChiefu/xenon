
<style>

  .chat {
    display: flex;
    flex-direction: column;
    flex: 1;
    overflow: hidden;
    min-width: 50%;
  }

  .header {
    display: flex;
    flex-direction: row;
    flex: none;
    padding: 0.5rem;
    align-items: center;
  }

  .header h3 {
    margin-right: auto;
    font-size: 1rem;
  }

  .messages {
    flex: 1;
    min-height: 0;
    display: flex;
    flex-direction: column;
    overflow-y: auto;
  }

  .composer {
    flex: none;
    display: flex;
    padding: 0.5rem;
    gap: 0.5rem;
    border-top: 1px solid var(--border);
  }

</style>

<div class="chat">

  <div class="header">
    <h3>Chat</h3>
  </div>

  <div class="messages"></div>

  <div class="composer">
    <input type="text" placeholder="Message"/>
  </div>
</div>
