# Sqlite Interface

## Auth

This is authenitication

Lets focus on small mvp:
(email user)
- invite user
    - if some verified contact exists


from invitation
- contact info is locked in
(screen name, password)
- profile which is:
    - id
    - screen name
    - hashed password
    - last_login
- contact info which is:
    - id
    - person_id
    - contact_type
    - contact_content
    - verified_at


dangerously_create_person:
(contact type, screen name, password)
    - person_id
    - screen name
    - contact type
    - contact content


The PROFILE id is a people_id. It's used as a reference across other systems.
It is a 64bit unsigend integer.

## Global audiences

Should be advised to start a system using a minimal number of logical volumes.
0-8191 could be interperated as a _global_ address. And logical volumes will correspond
to physical volumes eventually. As in, the address must be saved somewhere.

There is a physicality to addresses. So if I start in california, I could reserve the
logical volumes for the americas at 0-1024 and leave 7/8 of the available logical
volumes for more densely populated regions.


- create person / screen name from invitation
- create contact info

- get user by screename
- get user by contact
- veryfy user by password
- verify user has role

Authorization
- create role
- set role to person
- create session
- verify session
- update session