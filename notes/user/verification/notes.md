user creation
	- create user
		- user.verified = 0 (initial)
			- if env var for USER_EMAIL_VERIFICATION_ENABLED == 1, set user.verified = 1 AND no email sent 
	
	- if user.verified == 0
		- send verification email  
		- set db values
			- user_verified.id
			- user_verified.user_id (fk to user.id)
			- user_verified.token
			- user_verified.email (unique in this table too)
			- user_verified.state = 0 (0 = unverified, 1 = verified)
			- user_verified.exp_date = now + USER_EMAIL_VERIFICATION_EXP_IN_SECONDS
			- user_verified.verify_date
			- user_verified.creation_date

user change username
	- if env var for USER_EMAIL_VERIFICATION_ENABLED == 0 
		- new verification email
			- unique email constraint passes
			- user.verified = 0 (initial)
			- user_verified.state = 0
			- user_verified.exp_date = now + USER_EMAIL_VERIFICATION_EXP_IN_SECONDS
			- user_verified.token = new token
			- user_verified.email = new user.email
	- else
		- user.verified = 1 (verified)
user login
	- check account status
	- check verify status
		- reject if user.verified = 0 (initial)
	- works if user.verified state = 1 (verified)

user clicks email url: /verify
	- if env var for USER_EMAIL_VERIFICATION_ENABLED != 0
		- return 200
	- public url
		- no jwt auth required (user is not verified so login will fail)
		- 2 query params on the url:
			- u=user_verified.id
			- t=user_verified.token
	- look up user by pk user_verified.id

	- verification checks:
		- if user.state == 0 => reject 401 with: disabled account
		- if user.verified == 1 => send back 200 with msg: already verified
		- if user_verified.exp_date > now => reject 401 with: unable to verify your email, the verification url has expired
		- if user_verified.token != token from url query param => reject 401 with: unable to verify your email with this token
	- set user to verified
		- user.verified = 1
		- user_verified.state = 1
		- user_verified.verify_date = now
	- return 200

