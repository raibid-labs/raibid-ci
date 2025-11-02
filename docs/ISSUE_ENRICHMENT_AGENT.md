# Issue Enrichment Agent Guide

> **Draft Issue Enrichment Workflow - Preparing Issues for Implementation**

## Overview

The Issue Enrichment Agent is a specialized agent that improves draft issues before implementation begins. This agent iterates with the issue creator to ensure all requirements are clear, complete, and actionable for development agents.

## When to Use

**Enrichment agents are spawned automatically when:**
- An issue is created or edited with the `draft` or `status:draft` label
- The issue needs clarification, structure, or additional context
- The issue creator wants to iterate on requirements before implementation

**Enrichment completes when:**
- The issue body has proper structure (Summary, Requirements, Acceptance Criteria)
- The issue has clear, testable acceptance criteria
- The draft label is removed by the enrichment agent
- The issue is ready for implementation (clarifying questions can be answered during development)

## Workflow

### 1. Draft Issue Created

```
User creates issue with "draft" label
    ↓
Enrichment workflow detects draft state
    ↓
Posts enrichment agent spawn trigger
    ↓
Orchestrator spawns enrichment agent
```

### 2. Enrichment Agent Tasks

The enrichment agent should:

#### Analyze Current State
- Read the issue title and body using `gh issue view <number>`
- Identify what's clear vs. unclear
- Note missing sections or information
- Assess completeness for implementation

#### Update Issue Body Directly
**IMPORTANT**: Update the issue body using `gh issue edit <number> --body "..."` to add structure:
- This ensures implementation agents see the enriched version in the issue description
- Use comments for clarifying questions and progress updates (supplementary)
- The issue body is the source of truth - must contain all requirements and acceptance criteria

#### Identify Gaps
- **Requirements**: Are all functional requirements specified?
- **Acceptance Criteria**: Are success conditions testable and specific?
- **Context**: Is there sufficient background information?
- **Dependencies**: Are dependencies on other work identified?
- **Scope**: Is the scope well-defined and reasonable?
- **Technical Constraints**: Are there platform/technology constraints?

#### Ask Clarifying Questions
Post comments with a `## Clarifying Questions` section:

```markdown
## Clarifying Questions

1. Which authentication method should be used (OAuth, JWT, API keys)?
2. Should the API support pagination? If so, what page sizes?
3. Are there rate limiting requirements?
4. Should errors return detailed messages or generic ones for security?
```

#### Suggest Structure
If missing, recommend adding sections:

```markdown
## Summary
One-paragraph overview of the issue

## Requirements
- Specific functional requirement 1
- Specific functional requirement 2
- Non-functional requirement (performance, security, etc.)

## Acceptance Criteria
- [ ] Specific, testable criterion 1
- [ ] Specific, testable criterion 2
- [ ] Tests pass
- [ ] Documentation updated

## Dependencies
- Depends on: #123, #456
- Blocks: #789

## Technical Notes
- Platform constraints
- Technology choices
- Architecture considerations
```

#### Enrich Content
- **Expand vague requirements** into specific ones
- **Suggest test scenarios** to validate requirements
- **Identify edge cases** that need handling
- **Recommend phasing** if scope is large
- **Flag risks** or complex areas
- **Suggest similar examples** from the codebase

### 3. Completion and Transition

```
Agent enriches issue body with structure
    ↓
Agent posts clarifying questions (optional, as comments)
    ↓
Agent removes draft label (enrichment complete)
    ↓
Normal implementation workflow begins
    ↓
If questions exist: Normal workflow adds "waiting:answers" label
    ↓
Development can proceed, or pause until questions answered
```

## Best Practices

### DO ✅

1. **Be Specific in Questions**
   - ❌ "How should this work?"
   - ✅ "Should the search be case-sensitive or case-insensitive?"

2. **Suggest Concrete Options**
   - ❌ "What should the API look like?"
   - ✅ "Should the API follow REST principles with GET /users/{id} or use GraphQL?"

3. **Identify Testability**
   - Add specific, measurable acceptance criteria
   - Suggest test scenarios for each requirement
   - Note what success looks like

4. **Flag Dependencies Early**
   - Identify which other issues must complete first
   - Note if existing code needs refactoring
   - Highlight integration points

5. **Maintain Issue Structure**
   - Keep sections organized and clear
   - Use consistent formatting
   - Add links to relevant documentation

6. **Iterate Gradually**
   - Focus on biggest gaps first
   - Don't overwhelm with too many questions
   - Build on previous answers

### DON'T ❌

1. **Don't Start Implementation**
   - This is preparation only, not coding
   - Don't write code or create PRs
   - Don't make architecture decisions unilaterally

2. **Don't Make Assumptions**
   - If unclear, ask - don't guess
   - Don't assume technology choices
   - Don't infer requirements not stated

3. **Don't Bikeshed**
   - Focus on meaningful gaps, not style
   - Don't debate naming until implementation
   - Keep it actionable, not theoretical

4. **Don't Block**
   - If one area is unclear, move to others
   - Don't wait for all answers before enriching
   - Make progress incrementally

## Example Enrichment Session

### Initial Draft Issue

```markdown
# Add User Authentication

We need authentication for the app.
```

### After Enrichment Agent (Iteration 1)

**Agent Comment:**

I've analyzed this draft issue and identified several areas that need clarification. I've updated the issue structure and added clarifying questions below.

**Updated Issue Body:**

```markdown
# Add User Authentication

## Summary
Implement user authentication to secure the application and identify users.

## Requirements (DRAFT - needs clarification)
- User login functionality
- Secure credential storage
- Session management
- [Need to clarify: logout, password reset, etc.]

## Clarifying Questions

1. **Authentication Method**: Should we use OAuth 2.0, JWT tokens, session cookies, or another method?
2. **Password Requirements**: Are there specific password complexity requirements (length, special characters, etc.)?
3. **Session Duration**: How long should user sessions last? Should there be "remember me" functionality?
4. **Multi-Factor Auth**: Is two-factor authentication (2FA) required?
5. **Social Login**: Should users be able to authenticate via Google, GitHub, or other providers?
6. **Account Recovery**: What's the process for password reset? Email-based, security questions, or admin reset?
7. **Rate Limiting**: Should we implement rate limiting on login attempts to prevent brute force attacks?
8. **Concurrent Sessions**: Can a user be logged in from multiple devices simultaneously?

## Acceptance Criteria (TO BE REFINED after questions answered)
- [ ] Users can securely authenticate
- [ ] Credentials are encrypted at rest
- [ ] Sessions expire appropriately
- [ ] All authentication tests pass
- [ ] Security best practices followed

## Technical Notes
- Consider using bcrypt for password hashing
- Need to choose between stateful (server sessions) vs. stateless (JWT) approach
- May impact database schema (users table)
```

### After User Answers

**User Comment:**

```
1. Use JWT tokens
2. Minimum 8 characters, at least one number
3. 24 hours, with refresh tokens for 30 days
4. Not required for MVP
5. Not needed for MVP
6. Email-based reset with time-limited token
7. Yes, 5 attempts per 15 minutes per IP
8. Yes, multiple concurrent sessions allowed
```

### After Enrichment Agent (Iteration 2)

**Agent Comment:**

Thank you for the answers! I've updated the issue with specific requirements based on your responses. The issue is now much clearer and ready for implementation once you remove the draft label.

**Final Issue Body:**

```markdown
# Add User Authentication with JWT

## Summary
Implement JWT-based authentication system with email/password login, password reset capability, and rate-limited login attempts.

## Requirements
- JWT token-based authentication (access + refresh tokens)
- Email/password user registration
- Secure login with bcrypt password hashing
- Access tokens valid for 24 hours
- Refresh tokens valid for 30 days
- Email-based password reset with time-limited tokens
- Rate limiting: 5 login attempts per IP per 15 minutes
- Support for multiple concurrent sessions per user

## Password Policy
- Minimum 8 characters
- Must contain at least one number
- Case-sensitive
- Stored using bcrypt with appropriate cost factor

## Acceptance Criteria
- [ ] Users can register with email/password
- [ ] Users can log in and receive JWT access + refresh tokens
- [ ] Tokens are validated on protected endpoints
- [ ] Access tokens expire after 24 hours
- [ ] Refresh tokens can generate new access tokens
- [ ] Refresh tokens expire after 30 days
- [ ] Password reset sends email with time-limited token (1 hour)
- [ ] Password reset validates token and allows new password
- [ ] Rate limiting blocks IPs after 5 failed attempts for 15 minutes
- [ ] Passwords are hashed with bcrypt (cost factor >= 10)
- [ ] All tests pass (unit + integration)
- [ ] Security audit completed (no secrets in code, proper error handling)
- [ ] API documentation updated

## API Endpoints (Suggested)
- `POST /auth/register` - Create new user account
- `POST /auth/login` - Authenticate and receive tokens
- `POST /auth/refresh` - Get new access token using refresh token
- `POST /auth/logout` - Invalidate refresh token
- `POST /auth/password-reset-request` - Request password reset email
- `POST /auth/password-reset-confirm` - Submit new password with token

## Dependencies
- Depends on: User model/database schema (#XX if exists)
- Depends on: Email service setup (#XX if exists)
- Blocks: All features requiring authentication

## Technical Considerations
- Use standard JWT library (e.g., jsonwebtoken)
- Store refresh tokens in database with user association
- Include user ID in JWT claims
- Use environment variables for JWT secret
- Implement middleware for protected routes
- Consider Redis for rate limiting storage
- Email templates needed for password reset

## Test Scenarios
1. **Happy Path**: Register → Login → Access protected resource → Refresh token → Logout
2. **Invalid Credentials**: Login with wrong password (should fail, count toward rate limit)
3. **Expired Token**: Use expired access token (should reject, require refresh)
4. **Concurrent Sessions**: Login from multiple devices (both should work)
5. **Rate Limiting**: 6 rapid failed logins (6th should be blocked)
6. **Password Reset**: Request reset → Receive email → Submit new password → Login with new password
7. **Invalid Reset Token**: Try reset with expired/invalid token (should fail)
```

### Enrichment Agent Removes Draft Label

**Agent Final Comment:**

Enrichment complete! I've updated the issue with:
- Clear requirements and acceptance criteria
- Technical approach and implementation notes
- Clarifying questions for refinement (optional - can be answered during implementation)

Removing draft label - issue is ready for implementation agent.

**Result:** Issue is now ready for implementation! The normal orchestrator workflow will:
- Detect clarifying questions (if any) and add "waiting:answers" label
- Or spawn development agent immediately if no questions
- Development agent will implement following TDD workflow

## Agent Response Examples

### Good Clarifying Question
```markdown
## Clarifying Question: API Pagination

I noticed the requirement mentions "list all users" but no pagination details.

**Question**: Should the API support pagination for the user list endpoint?

**Options to consider**:
- A) Cursor-based pagination (recommended for large datasets)
- B) Offset-based pagination (simpler, may have performance issues at high offsets)
- C) No pagination (return all users - only suitable for small user bases)

**Additional questions if pagination is needed**:
- What should the default page size be?
- What should the maximum page size be?
- Should the response include pagination metadata (total count, has_more, etc.)?
```

### Good Enrichment Suggestion
```markdown
I've reviewed the acceptance criteria and notice they're quite high-level. Here are more specific, testable criteria I recommend:

**Current**: "Search should work correctly"

**Suggested Replacement**:
- [ ] Search returns exact matches first, then partial matches
- [ ] Search is case-insensitive
- [ ] Search handles special characters without errors
- [ ] Search returns results within 200ms for datasets up to 10,000 items
- [ ] Empty search query returns all items (up to pagination limit)
- [ ] Search with no matches returns empty array (not error)
- [ ] Search terms are trimmed of leading/trailing whitespace
```

## Labels Used

- **`draft`** or **`status:draft`** - Issue is in draft state for enrichment
- **`enrichment:active`** - Enrichment agent is currently working on this issue
- **`waiting:answers`** - Clarifying questions need answering (added by enrichment agent)

## Transition to Implementation

When the issue creator removes the `draft` label:

1. **Enrichment workflow** posts completion message
2. **Enrichment label** (`enrichment:active`) is removed
3. **Normal issue workflow** checks for remaining clarifying questions
4. **If no questions or all answered**: Development agent is spawned
5. **If questions remain**: Issue enters `waiting:answers` state until answered

## Tips for Issue Creators

### To Get Best Results

1. **Start with what you know** - Don't wait for perfect clarity
2. **Use draft label early** - Get enrichment help before coding starts
3. **Respond to questions promptly** - Faster iteration means faster delivery
4. **Provide context** - Link to similar features, explain the "why" not just "what"
5. **Remove draft when satisfied** - Trust that enrichment is complete enough

### Example Starting Template

```markdown
# [Feature Name]

**draft** (add this label)

## What I Want
Brief description of the feature or fix needed.

## Why
Business value or problem this solves.

## Ideas/Constraints
- Any initial thoughts
- Technical constraints known
- Similar features to reference

[Let enrichment agent fill in the rest!]
```

## Orchestrator Integration

The enrichment agent system integrates with the orchestrator via:

- **Workflow**: `.github/workflows/orchestrator-draft-enrichment.yml`
- **Detection Script**: `.github/scripts/check-draft-status.sh`
- **Spawn Trigger**: Comment containing `ORCHESTRATOR-SPAWN-ENRICHMENT-AGENT`
- **State Marker**: Comment containing `ENRICHMENT-AGENT-ACTIVE`

The orchestrator will:
1. Detect draft-labeled issues
2. Spawn enrichment agent
3. Skip normal implementation workflow while draft
4. Transition to implementation workflow when draft removed

## Related Documentation

- **Main Orchestrator Guide**: [ORCHESTRATOR.md](../ORCHESTRATOR.md)
- **Event-Driven Architecture**: [docs/architecture/event-driven-orchestration.md](architecture/event-driven-orchestration.md)
- **Agent Patterns**: [docs/ORCHESTRATOR_AGENT.md](ORCHESTRATOR_AGENT.md)

---

**Status**: Active
**Version**: 1.0
**Last Updated**: 2025-10-30
