# Due dates & overdue filtering

## User story

As a student planning my deployment tasks,
I want to attach a due date to each post,
so that I can see which tasks are overdue and prioritise them.

## Glossary

- **due_date** — an optional calendar date (`YYYY-MM-DD`, no time, no timezone).
- **today** — the current calendar date in UTC on the server.
- **past** — strictly before today. A due_date equal to today is *not* past.
- **overdue post** — a post whose due_date is past AND whose `published` is `false`.
- **pending post** — a post whose `published` is `false` and that is not overdue
  (due today or later, or with no due_date).

## Requirements (EARS)

### Creating posts

- **FR1:** When a client creates a post with a due_date of today or later, the
  system shall store the post and return `201` with the due_date in the response body.
- **FR2:** If a client creates a post with a due_date in the past, the system
  shall reject it with `400` and the error message `due_date must not be in the past`.
- **FR3:** When a client creates a post without a due_date, the system shall
  store the post with no due date and return `201` with `due_date` as `null`.

### Filtering the list

- **FR4:** When a client requests `GET /api/posts?status=overdue`, the system
  shall return only overdue posts.
- **FR5:** When a client requests `GET /api/posts?status=pending`, the system
  shall return only pending posts.
- **FR6:** When a client requests `GET /api/posts?status=published`, the system
  shall return only posts whose `published` is `true`.

### Sad paths added after critique

- **FR7:** If a client requests `GET /api/posts` with any other `status` value,
  the system shall reject it with `400` and the error message
  `status must be one of overdue, pending, published, all`.
- **FR8:** If a client sends a due_date that is not a valid `YYYY-MM-DD` date,
  the system shall reject the request with `400`.
- **FR9:** If a client updates a post with a due_date in the past, the system
  shall reject it with `400` and the error message `due_date must not be in the past`.
- **FR10:** When a client requests `GET /api/posts?status=all` or omits
  `status`, the system shall return all posts.

## Non-functional / constraints

- **NFR1:** Existing clients that never send `due_date` or `status` shall see
  no change in behaviour (constitution: no breaking changes).
- **NFR2:** The `status` filter shall be applied before pagination, so page
  counts reflect the filtered set.

## Decisions (from clarify)

| Question | Decision | Why |
|---|---|---|
| Past due date: reject or clamp to today? | Reject with `400` | Silently changing input hides client bugs |
| Is a due_date of today past? | No, today is allowed | A task due today is not late yet |
| Date or timestamp? Timezone? | Calendar date, compared against today in UTC | Deadlines are day-level; UTC keeps container and tests consistent |
| Do undated posts appear as overdue? | Never; they count as pending if unpublished | No deadline means it cannot be late |
| Can a published post be overdue? | No | Published means done |
| Does a past due_date block updates to a post that is already overdue? | Only if the update sends a past due_date (FR9) | Keeps one rule for create and update |
| Does `PUT` without `due_date` keep or clear it? | Clears it (sets `null`) | `PUT` already replaces the whole post |
| Unknown `status` value? | `400` (FR7), not silently "all" | A typo should be visible, not ignored |

## Out of scope

- Reminders or notifications for upcoming due dates
- Sorting by due_date
- Due dates with a time of day or per-user timezones
- Changes to the HTML views under `/posts` (API only)

## Critique log

**Iteration 1** — prompt: *"Review this requirements doc for: ambiguity,
missing sad paths, untestable statements, scope creep. List issues; do not
rewrite."*

Issues found in the first draft (FR1–FR6 only):

1. **Ambiguity:** "in the past" was undefined; a due_date of today could be read
   either way, and no timezone was given. → Fixed: added the Glossary (today =
   UTC date, past = strictly before today).
2. **Missing sad path:** an unknown `status` value had no defined behaviour. →
   Fixed: added FR7.
3. **Missing sad path:** nothing covered a malformed date such as `"tomorrow"` or
   `"2026-13-45"`. → Fixed: added FR8.
4. **Missing sad path:** no requirement covered updating a post's due_date, so a
   client could bypass FR2 through `PUT`. → Fixed: added FR9.
5. **Ambiguity:** "overdue" did not say whether undated or published posts count.
   → Fixed: defined overdue and pending in the Glossary.
6. **Untestable:** FR4 said "the system shall show overdue posts" without saying
   what else is excluded. → Fixed: FR4–FR6 now say "only".
7. **One shall per requirement:** FR6 covered both `published` and `all`. →
   Fixed: split `all`/omitted into FR10.
8. **Interaction not specified:** filtering vs pagination order. → Fixed: added NFR2.
