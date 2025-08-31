<script lang="ts">
    export let data: string | Record<string, any>;

    /**
     * Taken from StackOverflow:
     * https://stackoverflow.com/questions/4810841/pretty-print-json-using-javascript
     */
    const syntaxHighlight = (json: string) => {
        json = json.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;");

        return json.replace(
            /("(\\u[a-zA-Z0-9]{4}|\\[^u]|[^\\"])*"(\s*:)?|\b(true|false|null)\b|-?\d+(?:\.\d*)?(?:[eE][+\-]?\d+)?)/g,
            function(match) {
                var cls = "number";
                if (/^"/.test(match)) {
                    if (/:$/.test(match)) {
                        cls = "key";
                    } else {
                        cls = "string";
                    }
                } else if (/true|false/.test(match)) {
                    cls = "boolean";
                } else if (/null/.test(match)) {
                    cls = "null";
                }

                return "<span class=\"" + cls + "\">" + match + "</span>";
            }
        );
    };

    $: displayDom = "<pre>" + syntaxHighlight(typeof data === "string" ? data : JSON.stringify(data, null, 4)) + "</pre>";
</script>

<div class="JsonView">
    {@html displayDom}
</div>

<style>
    .JsonView {
        & pre {
            margin-top: 0;
        }

        font-size: 0.8em;
        line-height: 1.4;

        & .string {
            color: var(--robin-egg-blue);
        }

        & .number {
            color: var(--text-normal);
        }

        & .boolean {
            color: var(--accent-color-brighter);
        }

        & .null {
            color: var(--alert-color);
        }

        & .key {
            color: var(--taupe-gray);
        }

        @media (prefers-color-scheme: light) {
            & .string {
                color: blue;
            }

            & .number {
                color: purple;
            }

            & .boolean {
                color: darkgreen;
            }

            & .null {
                color: red;
            }

            & .key {
                color: dimgray;
            }
        }
    }
</style>