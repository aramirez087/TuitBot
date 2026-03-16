# Page snapshot

```yaml
- generic [ref=e4]:
  - generic [ref=e5]:
    - img [ref=e6]
    - heading "Tuitbot" [level=1] [ref=e7]
  - paragraph [ref=e8]: Enter your passphrase to access the dashboard.
  - generic [ref=e9]:
    - generic [ref=e10]:
      - generic [ref=e11]: Passphrase
      - textbox "Passphrase" [ref=e12]:
        - /placeholder: word1 word2 word3 word4
        - text: test test test test
    - alert [ref=e13]: Cannot reach the server. Is it running?
    - button "Sign in to Tuitbot" [ref=e14] [cursor=pointer]:
      - img [ref=e15]
      - text: Sign in
  - generic [ref=e16]:
    - paragraph [ref=e17]: Forgot your passphrase?
    - paragraph [ref=e18]: "Reset it from the terminal:"
    - code [ref=e19]: tuitbot-server --reset-passphrase
    - paragraph [ref=e20]: "Or if using cargo:"
    - code [ref=e21]: cargo run -p tuitbot-server -- --reset-passphrase
  - paragraph [ref=e22]:
    - text: Start the server with
    - code [ref=e23]: "--host 0.0.0.0"
    - text: to access from other devices on your network.
```