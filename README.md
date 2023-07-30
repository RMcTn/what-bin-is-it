```
brew install geckodriver
```

Run ```geckodriver``` before running the program


## Required ENV vars
AWS_ACCESS_KEY_ID  
AWS_SECRET_ACCESS_KEY  
TO_EMAIL_ADDRESS  
FROM_EMAIL_ADDRESS  
POSTCODE  
HOME_ADDRESS  

## Cross compile
```cargo install cross --git https://github.com/cross-rs/cross```
```cross build --target aarch64-unknown-linux-gnu```
