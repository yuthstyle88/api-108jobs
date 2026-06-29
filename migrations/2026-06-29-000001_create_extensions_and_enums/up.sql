CREATE SCHEMA utils;

CREATE EXTENSION IF NOT EXISTS ltree WITH SCHEMA public;

CREATE EXTENSION IF NOT EXISTS pg_trgm WITH SCHEMA public;

CREATE EXTENSION IF NOT EXISTS pgcrypto WITH SCHEMA public;

CREATE SEQUENCE public.changeme_seq
    START WITH 1
    INCREMENT BY 1
    NO MINVALUE
    NO MAXVALUE
    CACHE 1
    CYCLE;

CREATE TYPE public.billing_status AS ENUM (
    'QuotePendingReview',
    'OrderApproved',
    'Canceled'
);

CREATE TYPE public.category_follower_state AS ENUM (
    'Accepted',
    'Pending',
    'ApprovalRequired'
);

CREATE TYPE public.category_visibility AS ENUM (
    'Public',
    'LocalOnlyPublic',
    'LocalOnlyPrivate',
    'Private',
    'Unlisted'
);

CREATE TYPE public.federation_mode_enum AS ENUM (
    'All',
    'Local',
    'Disable'
);

CREATE TYPE public.intended_use_enum AS ENUM (
    'Business',
    'Personal',
    'Unknown'
);

CREATE TYPE public.job_type_enum AS ENUM (
    'Freelance',
    'Contract',
    'PartTime',
    'FullTime'
);

CREATE TYPE public.listing_type_enum AS ENUM (
    'All',
    'Local',
    'Subscribed',
    'ModeratorView'
);

CREATE TYPE public.payment_method AS ENUM (
    'Cash',
    'Coin'
);

CREATE TYPE public.post_kind AS ENUM (
    'Normal',
    'Delivery',
    'RideTaxi'
);

CREATE TYPE public.post_listing_mode_enum AS ENUM (
    'List',
    'Card',
    'SmallCard'
);

CREATE TYPE public.post_notifications_mode_enum AS ENUM (
    'RepliesAndMentions',
    'AllProposals',
    'Mute'
);

CREATE TYPE public.post_sort_type_enum AS ENUM (
    'Active',
    'Hot',
    'New',
    'Old',
    'Top',
    'MostProposals',
    'NewProposals',
    'Controversial',
    'Scaled'
);

CREATE TYPE public.proposal_sort_type_enum AS ENUM (
    'Hot',
    'Top',
    'New',
    'Old',
    'Controversial'
);

CREATE TYPE public.registration_mode_enum AS ENUM (
    'Closed',
    'RequireApplication',
    'Open'
);

CREATE TYPE public.rider_verification_status AS ENUM (
    'Pending',
    'Verified',
    'Rejected'
);

CREATE TYPE public.top_up_status AS ENUM (
    'Pending',
    'Success',
    'Expired'
);

CREATE TYPE public.trip_status AS ENUM (
    'Pending',
    'Assigned',
    'RiderConfirmed',
    'EnRouteToPickup',
    'PickedUp',
    'EnRouteToDropoff',
    'Delivered',
    'Cancelled'
);

CREATE TYPE public.tx_kind AS ENUM (
    'Deposit',
    'Withdraw',
    'Transfer'
);

CREATE TYPE public.vehicle_type AS ENUM (
    'Motorcycle',
    'Bicycle',
    'Car'
);

CREATE TYPE public.vote_show_enum AS ENUM (
    'Show',
    'ShowForOthers',
    'Hide'
);

CREATE TYPE public.withdraw_status AS ENUM (
    'Pending',
    'Rejected',
    'Completed',
    'Cancelled'
);

CREATE TYPE public.workflow_status AS ENUM (
    'WaitForFreelancerQuotation',
    'QuotationPendingReview',
    'OrderApproved',
    'InProgress',
    'PendingEmployerReview',
    'Completed',
    'Cancelled'
);

CREATE FUNCTION public.diesel_manage_updated_at(_tbl regclass) RETURNS void
    LANGUAGE plpgsql
    AS $$
BEGIN
EXECUTE format('CREATE TRIGGER set_updated_at BEFORE UPDATE ON %s
                    FOR EACH ROW EXECUTE PROCEDURE diesel_set_updated_at()', _tbl);
END;
$$;

CREATE FUNCTION public.diesel_set_updated_at() RETURNS trigger
    LANGUAGE plpgsql
    AS $$
BEGIN
    IF (NEW IS DISTINCT FROM OLD AND NEW.updated_at IS NOT DISTINCT FROM OLD.updated_at) THEN
        NEW.updated_at := CURRENT_TIMESTAMP;
END IF;
RETURN NEW;
END;
$$;

CREATE FUNCTION public.drop_ccnew_indexes() RETURNS integer
    LANGUAGE plpgsql
    AS $$
DECLARE
    i RECORD;
BEGIN
    FOR i IN (
        SELECT
            relname
        FROM
            pg_class
        WHERE
            relname LIKE '%ccnew%')
        LOOP
            EXECUTE 'DROP INDEX ' || i.relname;
        END LOOP;
    RETURN 1;
END;
$$;

CREATE FUNCTION public.generate_unique_changeme() RETURNS text
    LANGUAGE sql
    AS $$
    SELECT
        'http://changeme.invalid/seq/' || nextval('changeme_seq')::text;
$$;

CREATE FUNCTION public.random_smallint() RETURNS smallint
    LANGUAGE sql PARALLEL RESTRICTED
    RETURN trunc(((random() * (65536)::double precision) - (32768)::double precision));

CREATE FUNCTION utils.restore_views(p_view_schema character varying, p_view_name character varying) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_curr record;
BEGIN
    FOR v_curr IN (
        SELECT
            ddl_to_run,
            id
        FROM
            utils.deps_saved_ddl
        WHERE
            view_schema = p_view_schema
            AND view_name = p_view_name
        ORDER BY
            id DESC)
            LOOP
                BEGIN
                    EXECUTE v_curr.ddl_to_run;
                    DELETE FROM utils.deps_saved_ddl
                    WHERE id = v_curr.id;
                EXCEPTION
                    WHEN OTHERS THEN
                        -- keep looping, but please check for errors or remove left overs to handle manually
                END;
    END LOOP;
END;

$$;

CREATE FUNCTION public.reverse_timestamp_sort(t timestamp with time zone) RETURNS bigint
    LANGUAGE plpgsql IMMUTABLE PARALLEL SAFE
    AS $$
BEGIN
    RETURN (-1000000 * EXTRACT(EPOCH FROM t))::bigint;
END;
$$;

CREATE FUNCTION utils.save_and_drop_views(p_view_schema name, p_view_name name) RETURNS void
    LANGUAGE plpgsql
    AS $$
DECLARE
    v_curr record;
BEGIN
    FOR v_curr IN (
        SELECT
            obj_schema,
            obj_name,
            obj_type
        FROM ( WITH RECURSIVE recursive_deps (
                obj_schema,
                obj_name,
                obj_type,
                depth
) AS (
                SELECT
                    p_view_schema::name,
                    p_view_name,
                    NULL::varchar,
                    0
                UNION
                SELECT
                    dep_schema::varchar,
                    dep_name::varchar,
                    dep_type::varchar,
                    recursive_deps.depth + 1
                FROM (
                    SELECT
                        ref_nsp.nspname ref_schema,
                        ref_cl.relname ref_name,
                        rwr_cl.relkind dep_type,
                        rwr_nsp.nspname dep_schema,
                        rwr_cl.relname dep_name
                    FROM
                        pg_depend dep
                        JOIN pg_class ref_cl ON dep.refobjid = ref_cl.oid
                        JOIN pg_namespace ref_nsp ON ref_cl.relnamespace = ref_nsp.oid
                        JOIN pg_rewrite rwr ON dep.objid = rwr.oid
                        JOIN pg_class rwr_cl ON rwr.ev_class = rwr_cl.oid
                        JOIN pg_namespace rwr_nsp ON rwr_cl.relnamespace = rwr_nsp.oid
                    WHERE
                        dep.deptype = 'n'
                        AND dep.classid = 'pg_rewrite'::regclass) deps
                    JOIN recursive_deps ON deps.ref_schema = recursive_deps.obj_schema
                        AND deps.ref_name = recursive_deps.obj_name
                WHERE (deps.ref_schema != deps.dep_schema
                    OR deps.ref_name != deps.dep_name))
            SELECT
                obj_schema,
                obj_name,
                obj_type,
                depth
            FROM
                recursive_deps
            WHERE
                depth > 0) t
        GROUP BY
            obj_schema,
            obj_name,
            obj_type
        ORDER BY
            max(depth) DESC)
            LOOP
                IF v_curr.obj_type = 'v' THEN
                    INSERT INTO utils.deps_saved_ddl (view_schema, view_name, ddl_to_run)
                    SELECT
                        p_view_schema,
                        p_view_name,
                        'CREATE VIEW ' || v_curr.obj_schema || '.' || v_curr.obj_name || ' AS ' || view_definition
                    FROM
                        information_schema.views
                    WHERE
                        table_schema = v_curr.obj_schema
                        AND table_name = v_curr.obj_name;
                    EXECUTE 'DROP VIEW' || ' ' || v_curr.obj_schema || '.' || v_curr.obj_name;
                END IF;
            END LOOP;
END;
$$;

SET default_tablespace = '';

SET default_table_access_method = heap;
