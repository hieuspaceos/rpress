<?php
	/*	
	*	Goodlayers Getting Start
	*	---------------------------------------------------------------------
	*	This file contains function for getting start page
	*	---------------------------------------------------------------------
	*/

	add_filter( 'wp_insert_term_data', 'gdlr_core_wp_insert_term_data', 10, 3 );
	if ( ! function_exists( 'gdlr_core_wp_insert_term_data' ) ) {
		function gdlr_core_wp_insert_term_data( $data, $taxonomy, $args ) {
			global $gdlr_core_import_term;
			if ( ! empty( $gdlr_core_import_term['slug'] ) && $gdlr_core_import_term['slug'] == $data['slug'] ) {
				if( empty(term_exists($gdlr_core_import_term['id'])) ){
					$data['term_id'] = $gdlr_core_import_term['id'];
				}
				
			}
			return $data;
		}
	}

	// only load this class on ajax request
	if( defined('DOING_AJAX') && DOING_AJAX && !empty($_POST['action']) && ($_POST['action'] == 'gdlr_core_demo_images_import' || $_POST['action'] == 'gdlr_core_demo_import') ){

		// load importer api
		require_once ABSPATH . 'wp-admin/includes/import.php';

		if ( ! class_exists( 'WP_Importer' ) ) {
			$class_wp_importer = ABSPATH . 'wp-admin/includes/class-wp-importer.php';
			if ( file_exists( $class_wp_importer ) )
				require $class_wp_importer;
		}
	

		// add new class
		if( !class_exists('gdlr_core_importer') ){
			class gdlr_core_importer{

				private $unprocessed_images = array();
				private $image_per_process = 5;

				private $processed_terms = array();
				private $omitted_navigation = array();
				private $processed_post = array();
				private $post_orphans = array();
				private $featured_images = array();
				private $url_remap = array();
				private $missing_menu_items = array();

				private $demo_option = array();
				private $xml;

				private $errors = array();

				function __construct( $settings = array() ){
					add_action('wp_ajax_gdlr_core_demo_import', array(&$this, 'gdlr_core_demo_import'));
					add_action('wp_ajax_gdlr_core_demo_images_import', array(&$this, 'gdlr_core_demo_images_import'));
				}

				// ajax action to set variable
				function gdlr_core_demo_import(){

					if( !check_ajax_referer('gdlr_core_demo_import', 'security', false) ){
						die(json_encode(array(
							'status' => 'failed',
							'head' => esc_html__('Invalid Nonce', 'goodlayers-core'),
							'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
						)));
					}

					$this->image_per_process = apply_filters('gdlr_core_importer_image_per_process', 5);

					// set demo opton
					$demo_options = apply_filters('gdlr_core_demo_options', array());
					$this->demo_option = $demo_options[$_POST['demo-id']];

					$this->import(); 
				}

				// bulk image import request
				function gdlr_core_demo_images_import(){

					if( !check_ajax_referer('gdlr_core_demo_import', 'security', false) ){
						die(json_encode(array(
							'console'=> esc_html__('Invalid nonce for process :' ,'goodlayers-core') . ' ' . $_POST['current_process']
						)));
					}

					$this->image_per_process = apply_filters('gdlr_core_importer_image_per_process', 5);

					// get option
					$this->get_import_option();

					// get array of the image to process
					$current_process = intval($_POST['current_process']);
					$current_images = array_slice($this->unprocessed_images, ($current_process * $this->image_per_process), $this->image_per_process);
					
					// import process
					set_time_limit(0);

					wp_suspend_cache_invalidation(true);
					$this->process_posts($current_images);
					wp_suspend_cache_invalidation(false);

					$this->backfill_attachment_urls();
					$this->remap_featured_images();

					wp_cache_flush();

					// seccessfully import
					$ret = array('status' => 'success');
					if( !empty($this->errors) ){
						$ret['console'] = gdlr_core_debug_object($this->errors);
					}

					die(json_encode($ret));

				}

				// start import process
				function import(){

					do_action('gdlr_core_before_import');

					if( !empty($_POST['theme-option']) && !empty($this->demo_option['theme-option']) ){
						$this->import_key_value($this->demo_option['theme-option']);
						do_action('gdlr_core_theme_option_filewrite');

						do_action('gdlr_core_import_additional_options', $_POST['demo-id']);
					}

					$this->fetch_file();

					set_time_limit(0);

					wp_defer_term_counting(true);
					wp_defer_comment_counting(true);

					wp_suspend_cache_invalidation(true);
					$this->process_categories();
					$this->process_tags();
					$this->process_terms();
					$this->process_posts($this->xml['posts'], false);					
					wp_suspend_cache_invalidation(false);

					// update incorrect/missing information in the DB
					$this->backfill_parents();
					$this->backfill_attachment_urls();
					$this->remap_featured_images();

					wp_cache_flush();

					// delete option and generate it again
					// ref: wp-include/taxonomy.php
					foreach( get_taxonomies() as $tax ){
						delete_option("{$tax}_children");
						_get_term_hierarchy($tax);
					}

					wp_defer_term_counting(false);
					wp_defer_comment_counting(false);

					// import widget
					if( !empty($_POST['widget']) && !empty($this->demo_option['widget']) ){
						$this->import_key_value($this->demo_option['widget']);
					}
					// set menu navigation
					if( !empty($_POST['navigation']) && !empty($this->demo_option['menu']) ){
						$menu_settings = array();
						foreach( $this->demo_option['menu'] as $themes_location => $menu_id ){
							if( !empty($this->processed_terms[intval($menu_id)]) ){
								if( is_array($this->processed_terms[intval($menu_id)]) ){
									$menu_settings[$themes_location] = $this->processed_terms[intval($menu_id)]['term_id'];
								}else{
									$menu_settings[$themes_location] = $this->processed_terms[intval($menu_id)];
								}
							}
						}
						set_theme_mod('nav_menu_locations', $menu_settings);		
					}
					// set homepage id
					if( !empty($this->demo_option['page']) ){
						if( !empty($this->processed_posts[intval($this->demo_option['page'])]) ){
							update_option('show_on_front', 'page');
							update_option('page_for_posts', 0);
							update_option('page_on_front', $this->processed_posts[intval($this->demo_option['page'])]);
						}
					}
					
					// success status
					$ret = array(
						'status' => 'success',
						'message' => '<i class="fa fa-check"></i>\'' . $this->demo_option['title'] . '\' ' . esc_html__('demo successfully imported', 'goodlayers-core')
					);
					if( !empty($this->unprocessed_images) ){
						$ret['status'] = 'process';
						$ret['head'] = esc_html__('Importing images. Please wait...', 'goodlayers-core');
						$ret['process'] = ceil(sizeof($this->unprocessed_images) / $this->image_per_process);

						$this->set_import_option();
					}
					
					// display error if any
					$ret['console'] = '';
					if( !empty($this->errors) ){
						$ret['console'] = gdlr_core_debug_object($this->errors);
					}
					$ret['console'] .= ' importing ' . $this->image_per_process . ' images per process. ';

					do_action('gdlr_core_after_succuessful_import', $_POST['demo-id']);
					
					die(json_encode($ret));
				}

				// get-set import option
				function get_import_option(){
					// $process_option = get_option('gdlr_core_demo_processed_' . $_POST['demo-id'], array());
					// $this->processed_post = empty($process_option['processed_post'])? array(): $process_option['processed_post'];
					// $this->featured_images = empty($process_option['featured_images'])? array(): $process_option['featured_images'];
					// $this->unprocessed_images = empty($process_option['unprocessed_images'])? array(): $process_option['unprocessed_images'];
					
					$this->processed_post = get_option('gdlr_core_demo_processed_post_' . $_POST['demo-id'], array());
					$this->featured_images = get_option('gdlr_core_demo_processed_feai_' . $_POST['demo-id'], array());
					
					$upci1 = get_option('gdlr_core_demo_processed_upci1_' . $_POST['demo-id'], array());
					$upci2 = get_option('gdlr_core_demo_processed_upci2_' . $_POST['demo-id'], array());
					$this->unprocessed_images = array_merge($upci1, $upci2);
				}
				function set_import_option(){
					// $process_option = array(
					// 	'processed_post' => $this->processed_post,
					// 	'featured_images' => $this->featured_images,
					// 	'unprocessed_images' => $this->unprocessed_images
					// );	
					// update_option('gdlr_core_demo_processed_' . $_POST['demo-id'], $process_option);
					
					update_option('gdlr_core_demo_processed_post_' . $_POST['demo-id'], $this->processed_post, false);
					update_option('gdlr_core_demo_processed_feai_' . $_POST['demo-id'], $this->featured_images, false);

					$half_size = intval(count($this->unprocessed_images) / 2);
					update_option('gdlr_core_demo_processed_upci1_' . $_POST['demo-id'], array_slice($this->unprocessed_images, 0, $half_size), false);
					update_option('gdlr_core_demo_processed_upci2_' . $_POST['demo-id'], array_slice($this->unprocessed_images, $half_size), false);

				}

				// fetch file
				function fetch_file(){

					// fetch the file
					if( !empty($this->demo_option['xml']) && is_file($this->demo_option['xml']) ){
						$parser = new GDLR_CORE_WXR_Parser();
						$xml = $parser->parse($this->demo_option['xml']);

						if( is_wp_error($this->xml) ){
							die(json_encode(array(
								'status' => 'failed',
								'head' => esc_html__('File parse Error', 'goodlayers-core'),
								'message'=> $xml->get_error_message() . ' ' . esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
							)));
						}else{
							$this->xml = $xml;
						}
					}else{
						die(json_encode(array(
							'status' => 'failed',
							'head' => esc_html__('File not exists', 'goodlayers-core'),
							'message'=> esc_html__('Please refresh the page and try again.' ,'goodlayers-core')
						)));
					}

				}

				// process data
				function process_categories(){

					// has no category
					if( empty($this->xml['categories']) ) return;

					// not importing post
					if( empty($_POST['post']) ){
						unset($this->xml['categories']);
						return;
					}

					// inserting category one by one
					foreach( $this->xml['categories'] as $cat ){
						
						// if exists, ignore it
						$term_id = term_exists($cat['category_nicename'], 'category');
						if( $term_id ){
							if( is_array($term_id) ) $term_id = $term_id['term_id'];
							if( isset($cat['term_id']) )
								$this->processed_terms[intval($cat['term_id'])] = (int) $term_id;
							continue;
						}

						global $gdlr_core_import_term;
						$gdlr_core_import_term = array(
							'slug' => $cat['category_nicename'],
							'id'   => $cat['term_id'],
						);

						// insert
						$category_parent = empty($cat['category_parent'])? 0 : category_exists($cat['category_parent']);
						$category_description = isset($cat['category_description'])? $cat['category_description'] : '';
						$catarr = array(
							'category_nicename' => $cat['category_nicename'],
							'category_parent' => $category_parent,
							'cat_name' => $cat['cat_name'],
							'category_description' => $category_description
						);

						$id = wp_insert_category($catarr);
						if( !is_wp_error($id) ){
							if( isset($cat['term_id']) )
								$this->processed_terms[intval($cat['term_id'])] = $id;
						}else{
							$this->errors['category'] = empty($this->errors['category'])? array(): $this->errors['category'];
							$this->errors['category'][$cat['category_nicename']] = $id->get_error_message();
							continue;
						}						
					}

					unset($this->xml['categories']);
				}
				function process_tags(){
					
					// has no tag
					if( empty($this->xml['tags']) ) return;

					// not importing post
					if( empty($_POST['post']) ){
						unset($this->xml['tags']);
						return;
					}

					foreach( $this->xml['tags'] as $tag ){

						// if exists, ignore it
						$term_id = term_exists($tag['tag_slug'], 'post_tag');
						if( $term_id ){
							if( is_array($term_id) ) $term_id = $term_id['term_id'];
							if( isset($tag['term_id']) )
								$this->processed_terms[intval($tag['term_id'])] = (int) $term_id;
							continue;
						}

						global $gdlr_core_import_term;
						$gdlr_core_import_term = array(
							'slug' => $tag['tag_slug'],
							'id'   => $tag['term_id'],
						);

						// insert
						$tag_desc = isset($tag['tag_description']) ? $tag['tag_description']: '';
						$tagarr = array('slug' => $tag['tag_slug'], 'description' => $tag_desc);

						$id = wp_insert_term($tag['tag_name'], 'post_tag', $tagarr);
						if( !is_wp_error($id) ){
							if( isset($tag['term_id']) )
								$this->processed_terms[intval($tag['term_id'])] = $id;
						}else{
							$this->errors['tag'] = empty($this->errors['tag'])? array(): $this->errors['tag'];
							$this->errors['tag'][$tag['tag_name']] = $id->get_error_message();
							continue;
						}
					}

					unset($this->xml['tags']);
				}
				function process_terms(){

					// has no term
					if( empty($this->xml['terms']) ) return;
					
					foreach( $this->xml['terms'] as $term ){

						// if not importing portfolio
						if( empty($_POST['portfolio']) && in_array($term['term_taxonomy'], array('portfolio_category', 'portfolio_tag')) ) 
							continue;

						// if not importing navigation
						if( empty($_POST['navigation']) && $term['term_taxonomy'] == 'nav_menu' )
							continue;

						// if exists, ignore it
						$term_id = term_exists($term['slug'], $term['term_taxonomy']);
						if( $term_id ){
							if( is_array($term_id) ) $term_id = $term_id['term_id'];
							if( isset($term['term_id']) ) 
								$this->processed_terms[intval($term['term_id'])] = (int) $term_id;
							if( 'nav_menu' == $term['term_taxonomy'] )
								$this->omitted_navigation[$term['slug']] = (int) $term_id;
							
							continue;
						}

						if( empty($term['term_parent']) ){
							$parent = 0;
						}else{
							$parent = term_exists($term['term_parent'], $term['term_taxonomy']);
							if( is_array($parent) ) $parent = $parent['term_id'];
						}
						$description = isset($term['term_description'])? $term['term_description']: '';
						$termarr = array('slug' => $term['slug'], 'description' => $description, 'parent' => intval($parent));

						global $gdlr_core_import_term;
						$gdlr_core_import_term = array(
							'slug' => $term['slug'],
							'id'   => $term['term_id'],
						);

						$id = wp_insert_term( $term['term_name'], $term['term_taxonomy'], $termarr );
						if( !is_wp_error($id) ){
							if( isset($term['term_id']) )
								$this->processed_terms[intval($term['term_id'])] = $id;
						}else{
							$this->errors['term'] = empty($this->errors['term'])? array(): $this->errors['term'];
							$this->errors['term'][$term['term_taxonomy']] = $id->get_error_message();
							continue;
						}
					}

					unset($this->xml['terms']);
				}
				function process_posts( $post_list, $import_image = true ){
					
					foreach( $post_list as $post ){

						// if post type not exists
						if( !post_type_exists($post['post_type']) ){
							$this->errors['post_type_not_exists'] = empty($this->errors['post_type_not_exists'])? array(): $this->errors['post_type_not_exists'];
							$this->errors['post_type_not_exists'][$post['post_type']] = true;
							continue;
						}

						// if not importing
						if( (empty($_POST['post']) && $post['post_type'] == 'post') || 
							(empty($_POST['portfolio']) && $post['post_type'] == 'portfolio') ||
							(empty($_POST['image']) && $post['post_type'] == 'attachment') ){
							continue;
						}

						if( isset($this->processed_posts[$post['post_id']]) && !empty($post['post_id']) )
							continue;

						if( $post['status'] == 'auto-draft' )
							continue;
	
						if( !$import_image && 'attachment' == $post['post_type'] ){
							$this->unprocessed_images[] = $post;
							continue;
						}

						// for menu item
						if( $post['post_type'] == 'nav_menu_item' ){
							if( !empty($_POST['navigation']) ){
								$this->process_menu_item( $post );
							}
							continue;
						}

						// insert post
						$post_exists = post_exists($post['post_title'], '', $post['post_date']);
						if( $post_exists && get_post_type($post_exists) == $post['post_type'] ){
							$post_id = $post_exists;
						}else{

							$post_parent = (int) $post['post_parent'];
							if( $post_parent ){

								// determind the parent local id
								if( isset($this->processed_posts[$post_parent]) ){
									$post_parent = $this->processed_posts[$post_parent];

								// save for future uses
								}else{
									$this->post_orphans[intval($post['post_id'])] = $post_parent;
									$post_parent = 0;
								}
							}

							$author = (int) get_current_user_id();

							$postdata = array(
								'import_id' => $post['post_id'], 'post_author' => $author, 'post_date' => $post['post_date'],
								'post_date_gmt' => $post['post_date_gmt'], 'post_content' => $post['post_content'],
								'post_excerpt' => $post['post_excerpt'], 'post_title' => $post['post_title'],
								'post_status' => $post['status'], 'post_name' => $post['post_name'],
								'comment_status' => $post['comment_status'], 'ping_status' => $post['ping_status'],
								'guid' => $post['guid'], 'post_parent' => $post_parent, 'menu_order' => $post['menu_order'],
								'post_type' => $post['post_type'], 'post_password' => $post['post_password']
							);

							$original_post_ID = $post['post_id'];

							if( 'attachment' == $postdata['post_type'] ){

								$remote_url = !empty($post['attachment_url'])? $post['attachment_url']: $post['guid'];
								
								// try to use _wp_attached file for upload folder placement to ensure the same location as the export site
								$postdata['upload_date'] = $post['post_date'];
								if( isset($post['postmeta']) ){
									foreach( $post['postmeta'] as $meta ){
										if( $meta['key'] == '_wp_attached_file' ){
											if( preg_match('%^[0-9]{4}/[0-9]{2}%', $meta['value'], $matches) )
												$postdata['upload_date'] = $matches[0];
											break;
										}
									}
								}

								$post_id = $this->process_attachment($postdata, $remote_url);
							}else{
								$post_id = wp_insert_post($postdata, true);
							}

							if( is_wp_error($post_id) ){
								$this->errors['post'] = empty($this->errors['post'])? array(): $this->errors['post'];
								$this->errors['post'][$post['post_id']] = $post_id->get_error_message();
								continue;
							}

							if( $post['is_sticky'] == 1 )
								stick_post($post_id);
						}

						// map pre-import ID to local ID
						$this->processed_posts[intval($post['post_id'])] = (int) $post_id;

						// assign term to posts
						if( !empty($post['terms']) ){

							$terms_to_set = array();
							foreach( $post['terms'] as $term ){
								// back compat with WXR 1.0 map 'tag' to 'post_tag'
								$taxonomy = ('tag' == $term['domain'])? 'post_tag': $term['domain'];

								$term_exists = term_exists($term['slug'], $taxonomy);
								$term_id = is_array($term_exists)? $term_exists['term_id']: $term_exists;
								if( !$term_id ){
									$t = wp_insert_term($term['name'], $taxonomy, array('slug' => $term['slug']));
									if( !is_wp_error($t) ){
										$term_id = $t['term_id'];
									}else{
										$this->errors['post_term'] = empty($this->errors['post_term'])? array(): $this->errors['post_term'];
										$this->errors['post_term'][$term['name']] = $t->get_error_message();
										continue;
									}
								}
								$terms_to_set[$taxonomy][] = intval($term_id);
							}

							foreach( $terms_to_set as $tax => $ids ){
								$tt_ids = wp_set_post_terms( $post_id, $ids, $tax );
								do_action( 'wp_import_set_post_terms', $tt_ids, $ids, $tax, $post_id, $post );
							}
							unset($post['terms'], $terms_to_set);
						}

						// donot import comments
						if( !empty($post['comments']) ){
							unset($post['comments']);
						}

						// add post meta
						if( !empty($post['postmeta']) ){
							foreach($post['postmeta'] as $meta){

								$key = $meta['key'];
								$value = maybe_unserialize($meta['value']);

								if( '_edit_last' == $key )
									continue;

								if( '_wp_attachment_metadata' == $key || '_wp_attached_file' == $key )
									continue;
								
								// if the post has a featured image, take note of this in case of remap
								if( '_thumbnail_id' == $key )
									$this->featured_images[$post_id] = (int) $value;


								update_post_meta($post_id, $key, $value);
							}
						}

					} // foreach

					unset($this->xml['posts']);
				}

				// attachment process
				function process_attachment($post, $url){
					if( !empty($post['guid']) ){
						unset($post['guid']);
					}

					require_once(ABSPATH . 'wp-admin/includes/file.php');
					require_once(ABSPATH . 'wp-admin/includes/media.php');
					require_once(ABSPATH . 'wp-admin/includes/image.php');

					// Get the file name and download it to the temp dir
					$tmp = download_url($url);
					if( is_wp_error($tmp) ) return $tmp;

					// Get the image name
					$file_array = array();
					$file_array['name'] = basename($url);
					$file_array['tmp_name'] = $tmp;

					// Check for upload errors
    				$id = media_handle_sideload($file_array, 0, '', $post);

					// If error storing permanently, unlink temp file
					if( is_wp_error($id) ){
						@unlink($file_array['tmp_name']);
						return $id;
					}

					return $id;
				}
				function process_attachment_old($post, $url){

					// if the URL is absolute, but does not contain address, then upload it assuming base_site_url
					if( preg_match('|^/[\w\W]+$|', $url) )
						$url = rtrim(esc_url($this->xml['base_url']), '/') . $url;

					$upload = $this->fetch_remote_file($url, $post);

					// return error
					if( is_wp_error($upload) )
						return $upload;

					// check file type
					if( $info = wp_check_filetype($upload['file']) ){
						$post['post_mime_type'] = $info['type'];
					}else{
						return new WP_Error('attachment_processing_error', esc_html__('Invalid file type', 'goodlayers-core'));
					}
					
					// insert attachment
					$post['guid'] = $upload['url'];
					$post_id = wp_insert_attachment($post, $upload['file']);
					wp_update_attachment_metadata($post_id, wp_generate_attachment_metadata($post_id, $upload['file']));

					// remap resized image URLs, works by stripping the extension and remapping the URL stub.
					if( preg_match('!^image/!', $info['type'] )){
						$parts = pathinfo($url);
						$name = basename($parts['basename'], ".{$parts['extension']}"); // PATHINFO_FILENAME in PHP 5.2

						$parts_new = pathinfo($upload['url']);
						$name_new = basename($parts_new['basename'], ".{$parts_new['extension']}");

						$this->url_remap[$parts['dirname'] . '/' . $name] = $parts_new['dirname'] . '/' . $name_new;
					}

					return $post_id;
				}
				function fetch_remote_file($url, $post){
					// extract the file name and extension from the url
					$file_name = basename($url);

					// get placeholder file in the upload dir with a unique, sanitized filename
					$upload = wp_upload_bits($file_name, 0, '', $post['upload_date']);
					if( $upload['error'] )
						return new WP_Error('upload_dir_error', $upload['error']);

					// fetch the remote url and write it to the placeholder file
					$response = wp_remote_get($url, array(
						'stream' => true,
						'timeout' => 120,
						'filename' => $upload['file']
					));

					// request failed
					if( is_wp_error($response) ){
						@unlink($upload['file']);
						return $response;
					}

					// make sure the fetch was successful
					$response_code = (int) wp_remote_retrieve_response_code($response);
					if ( $response_code != '200' ) {
						@unlink( $upload['file'] );
						return new WP_Error('import_file_error', sprintf(esc_html__('Remote server returned error response %1$d %2$s for %3$s', 'goodlayers-core'), esc_html($response_code), get_status_header_desc($response_code), esc_html($url)));
					}

					$filesize = filesize($upload['file']);

					if( isset($headers['content-length']) && $filesize != $headers['content-length'] ){
						@unlink( $upload['file'] );
						return new WP_Error('import_file_error', esc_html__('Remote file is incorrect size', 'goodlayers-core'));
					}

					if( 0 == $filesize ){
						@unlink( $upload['file'] );
						return new WP_Error('import_file_error', esc_html__('Zero size file downloaded', 'goodlayers-core'));
					}

					// keep track of the old and new urls so we can substitute them later
					$this->url_remap[$url] = $upload['url'];
					$this->url_remap[$post['guid']] = $upload['url']; // r13735, really needed?

					// keep track of the destination if the remote url is redirected somewhere else
					if( isset($headers['x-final-location']) && $headers['x-final-location'] != $url )
						$this->url_remap[$headers['x-final-location']] = $upload['url'];
	
					return $upload;
				}

				// navigation process
				function process_menu_item($item){

					if( 'draft' == $item['status'] )
						return;

					if( !empty($this->processed_menu_items[intval($item['post_id'])]) )
						return;

					// get menu slug
					if( isset($item['terms']) ){
						foreach( $item['terms'] as $term ){
							if ( 'nav_menu' == $term['domain'] ){
								$menu_slug = $term['slug'];
								break;
							}
						}
					}
					if( empty($menu_slug) ) return;

					// if already imported this menu list once, ignore it
					if( !empty($this->omitted_navigation[$menu_slug]) ) return;

					// get menu id from slug
					$menu_id = term_exists($menu_slug, 'nav_menu');
					if( empty($menu_id) ){
						return;
					}else{
						$menu_id = is_array($menu_id)? $menu_id['term_id']: $menu_id;
					}

					foreach ( $item['postmeta'] as $meta ){
						${$meta['key']} = $meta['value'];
					}

					if( 'taxonomy' == $_menu_item_type && isset($this->processed_terms[intval($_menu_item_object_id)]) ){
						if( is_array($this->processed_terms[intval($_menu_item_object_id)]) ){
							$_menu_item_object_id = $this->processed_terms[intval($_menu_item_object_id)]['term_id'];
						}else{
							$_menu_item_object_id = $this->processed_terms[intval($_menu_item_object_id)];
						}
					}else if( 'post_type' == $_menu_item_type && isset( $this->processed_posts[intval($_menu_item_object_id)]) ){
						$_menu_item_object_id = $this->processed_posts[intval($_menu_item_object_id)];
					}else if( 'custom' != $_menu_item_type ){
						// associated object is missing or not imported yet, we'll retry later
						$this->missing_menu_items[] = $item;
						return;
					}

					if( isset($this->processed_menu_items[intval($_menu_item_menu_item_parent)]) ){
						$_menu_item_menu_item_parent = $this->processed_menu_items[intval($_menu_item_menu_item_parent)];
					}else if( $_menu_item_menu_item_parent ){
						$this->menu_item_orphans[intval($item['post_id'])] = (int) $_menu_item_menu_item_parent;
						$_menu_item_menu_item_parent = 0;
					}

					// wp_update_nav_menu_item expects CSS classes as a space separated string
					$_menu_item_classes = maybe_unserialize($_menu_item_classes);
					if ( is_array($_menu_item_classes) )
						$_menu_item_classes = implode(' ', $_menu_item_classes);

					$args = array(
						'menu-item-object-id' => $_menu_item_object_id,
						'menu-item-object' => $_menu_item_object,
						'menu-item-parent-id' => $_menu_item_menu_item_parent,
						'menu-item-position' => intval( $item['menu_order'] ),
						'menu-item-type' => $_menu_item_type,
						'menu-item-title' => $item['post_title'],
						'menu-item-url' => $_menu_item_url,
						'menu-item-description' => $item['post_content'],
						'menu-item-attr-title' => $item['post_excerpt'],
						'menu-item-target' => $_menu_item_target,
						'menu-item-classes' => $_menu_item_classes,
						'menu-item-xfn' => $_menu_item_xfn,
						'menu-item-status' => $item['status'],
						'import_id' => $item['post_id']
					);

					$id = $this->wp_update_nav_menu_item($menu_id, 0, $args);
					if( $id && !is_wp_error($id) )
						$this->processed_menu_items[intval($item['post_id'])] = (int) $id;

					// add custom navigation meta
					$custom_meta = apply_filters('gdlr_core_nav_menu_import_slug', array('gdlr_core_nav_menu_custom'));
					foreach( $custom_meta as $key ) {
						if( !empty($$key) ){
							$meta_value = maybe_unserialize($$key);
							update_post_meta((int) $id, $key, $meta_value);
						}
					}
				}

				// from wp-includes/nav-menu.php
				// add the import id before process the post
				function wp_update_nav_menu_item( $menu_id = 0, $menu_item_db_id = 0, $menu_item_data = array() ) {
					$menu_id = (int) $menu_id;
					$menu_item_db_id = (int) $menu_item_db_id;

					// make sure that we don't convert non-nav_menu_item objects into nav_menu_item objects
					if ( ! empty( $menu_item_db_id ) && ! is_nav_menu_item( $menu_item_db_id ) )
						return new WP_Error( 'update_nav_menu_item_failed', esc_html__( 'The given object ID is not that of a menu item.', 'goodlayers-core' ) );

					$menu = wp_get_nav_menu_object( $menu_id );

					if ( ! $menu && 0 !== $menu_id ) {
						return new WP_Error( 'invalid_menu_id', esc_html__( 'Invalid menu ID.', 'goodlayers-core' ) );
					}

					if ( is_wp_error( $menu ) ) {
						return $menu;
					}

					$defaults = array(
						'menu-item-db-id' => $menu_item_db_id,
						'menu-item-object-id' => 0,
						'menu-item-object' => '',
						'menu-item-parent-id' => 0,
						'menu-item-position' => 0,
						'menu-item-type' => 'custom',
						'menu-item-title' => '',
						'menu-item-url' => '',
						'menu-item-description' => '',
						'menu-item-attr-title' => '',
						'menu-item-target' => '',
						'menu-item-classes' => '',
						'menu-item-xfn' => '',
						'menu-item-status' => '',
					);

					$args = wp_parse_args( $menu_item_data, $defaults );

					if ( 0 == $menu_id ) {
						$args['menu-item-position'] = 1;
					} elseif ( 0 == (int) $args['menu-item-position'] ) {
						$menu_items = 0 == $menu_id ? array() : (array) wp_get_nav_menu_items( $menu_id, array( 'post_status' => 'publish,draft' ) );
						$last_item = array_pop( $menu_items );
						$args['menu-item-position'] = ( $last_item && isset( $last_item->menu_order ) ) ? 1 + $last_item->menu_order : count( $menu_items );
					}

					$original_parent = 0 < $menu_item_db_id ? get_post_field( 'post_parent', $menu_item_db_id ) : 0;

					if ( 'custom' != $args['menu-item-type'] ) {
						/* if non-custom menu item, then:
							* use original object's URL
							* blank default title to sync with original object's
						*/

						$args['menu-item-url'] = '';

						$original_title = '';
						if ( 'taxonomy' == $args['menu-item-type'] ) {
							$original_parent = get_term_field( 'parent', $args['menu-item-object-id'], $args['menu-item-object'], 'raw' );
							$original_title = get_term_field( 'name', $args['menu-item-object-id'], $args['menu-item-object'], 'raw' );
						} elseif ( 'post_type' == $args['menu-item-type'] ) {

							$original_object = get_post( $args['menu-item-object-id'] );
							$original_parent = (int) $original_object->post_parent;
							$original_title = $original_object->post_title;
						} elseif ( 'post_type_archive' == $args['menu-item-type'] ) {
							$original_object = get_post_type_object( $args['menu-item-object'] );
							if ( $original_object ) {
								$original_title = $original_object->labels->archives;
							}
						}

						if ( $args['menu-item-title'] == $original_title )
							$args['menu-item-title'] = '';

						// hack to get wp to create a post object when too many properties are empty
						if ( '' ==  $args['menu-item-title'] && '' == $args['menu-item-description'] )
							$args['menu-item-description'] = ' ';
					}

					// Populate the menu item object
					$post = array(
						'menu_order' => $args['menu-item-position'],
						'ping_status' => 0,
						'post_content' => $args['menu-item-description'],
						'post_excerpt' => $args['menu-item-attr-title'],
						'post_parent' => $original_parent,
						'post_title' => $args['menu-item-title'],
						'post_type' => 'nav_menu_item',
					);
					if( !empty($menu_item_data['import_id']) ){
						$post['import_id'] = $menu_item_data['import_id'];
					}

					$update = 0 != $menu_item_db_id;

					// New menu item. Default is draft status
					if ( ! $update ) {
						$post['ID'] = 0;
						$post['post_status'] = 'publish' == $args['menu-item-status'] ? 'publish' : 'draft';
						$menu_item_db_id = wp_insert_post( $post );
						if ( ! $menu_item_db_id	|| is_wp_error( $menu_item_db_id ) )
							return $menu_item_db_id;

						/**
						 * Fires immediately after a new navigation menu item has been added.
						 *
						 * @since 4.4.0
						 *
						 * @see wp_update_nav_menu_item()
						 *
						 * @param int   $menu_id         ID of the updated menu.
						 * @param int   $menu_item_db_id ID of the new menu item.
						 * @param array $args            An array of arguments used to update/add the menu item.
						 */
						do_action( 'wp_add_nav_menu_item', $menu_id, $menu_item_db_id, $args );
					}

					// Associate the menu item with the menu term
					// Only set the menu term if it isn't set to avoid unnecessary wp_get_object_terms()
					 if ( $menu_id && ( ! $update || ! is_object_in_term( $menu_item_db_id, 'nav_menu', (int) $menu->term_id ) ) ) {
						wp_set_object_terms( $menu_item_db_id, array( $menu->term_id ), 'nav_menu' );
					}

					if ( 'custom' == $args['menu-item-type'] ) {
						$args['menu-item-object-id'] = $menu_item_db_id;
						$args['menu-item-object'] = 'custom';
					}

					$menu_item_db_id = (int) $menu_item_db_id;

					update_post_meta( $menu_item_db_id, '_menu_item_type', sanitize_key($args['menu-item-type']) );
					update_post_meta( $menu_item_db_id, '_menu_item_menu_item_parent', strval( (int) $args['menu-item-parent-id'] ) );
					update_post_meta( $menu_item_db_id, '_menu_item_object_id', strval( (int) $args['menu-item-object-id'] ) );
					update_post_meta( $menu_item_db_id, '_menu_item_object', sanitize_key($args['menu-item-object']) );
					update_post_meta( $menu_item_db_id, '_menu_item_target', sanitize_key($args['menu-item-target']) );

					$args['menu-item-classes'] = array_map( 'sanitize_html_class', explode( ' ', $args['menu-item-classes'] ) );
					$args['menu-item-xfn'] = implode( ' ', array_map( 'sanitize_html_class', explode( ' ', $args['menu-item-xfn'] ) ) );
					update_post_meta( $menu_item_db_id, '_menu_item_classes', $args['menu-item-classes'] );
					update_post_meta( $menu_item_db_id, '_menu_item_xfn', $args['menu-item-xfn'] );
					update_post_meta( $menu_item_db_id, '_menu_item_url', esc_url_raw($args['menu-item-url']) );

					if ( 0 == $menu_id )
						update_post_meta( $menu_item_db_id, '_menu_item_orphaned', (string) time() );
					elseif ( get_post_meta( $menu_item_db_id, '_menu_item_orphaned' ) )
						delete_post_meta( $menu_item_db_id, '_menu_item_orphaned' );

					// Update existing menu item. Default is publish status
					if ( $update ) {
						$post['ID'] = $menu_item_db_id;
						$post['post_status'] = 'draft' == $args['menu-item-status'] ? 'draft' : 'publish';
						wp_update_post( $post );
					}

					/**
					 * Fires after a navigation menu item has been updated.
					 *
					 * @since 3.0.0
					 *
					 * @see wp_update_nav_menu_item()
					 *
					 * @param int   $menu_id         ID of the updated menu.
					 * @param int   $menu_item_db_id ID of the updated menu item.
					 * @param array $args            An array of arguments used to update a menu item.
					 */
					do_action( 'wp_update_nav_menu_item', $menu_id, $menu_item_db_id, $args );

					return $menu_item_db_id;
				}

				// parent data process
				function backfill_parents(){
					
					global $wpdb;

					// find parents for post orphans
					if( !empty($this->post_orphans) ){
						foreach( $this->post_orphans as $child_id => $parent_id ){
							$local_child_id = $local_parent_id = false;

							if( isset($this->processed_posts[$child_id]) )
								$local_child_id = $this->processed_posts[$child_id];
							if( isset( $this->processed_posts[$parent_id]) )
								$local_parent_id = $this->processed_posts[$parent_id];
							if( $local_child_id && $local_parent_id )
								$wpdb->update( $wpdb->posts, array('post_parent' => $local_parent_id), array('ID' => $local_child_id), '%d', '%d' );
						}
					}

					// all other posts/terms are imported, retry menu items with missing associated object
					if( !empty($this->missing_menu_items) ){
						foreach( $this->missing_menu_items as $item ){
							$this->process_menu_item($item);
						}
					}

					// find parents for menu item orphans
					if( !empty($this->menu_item_orphans) ){
						foreach ( $this->menu_item_orphans as $child_id => $parent_id ) {
							$local_child_id = $local_parent_id = 0;
						
							if( isset($this->processed_menu_items[$child_id]) )
								$local_child_id = $this->processed_menu_items[$child_id];
							if( isset($this->processed_menu_items[$parent_id]) )
								$local_parent_id = $this->processed_menu_items[$parent_id];
							if( $local_child_id && $local_parent_id )
								update_post_meta($local_child_id, '_menu_item_menu_item_parent', (int) $local_parent_id);
						}
					}

				}
				function backfill_attachment_urls(){
					global $wpdb;

					// make sure we do the longest urls first, in case one is a substring of another
					uksort($this->url_remap, array(&$this, 'cmpr_strlen'));

					foreach( $this->url_remap as $from_url => $to_url ){

						// remap urls in post_content
						$wpdb->query( $wpdb->prepare("UPDATE {$wpdb->posts} SET post_content = REPLACE(post_content, %s, %s)", $from_url, $to_url) );
						
						// remap enclosure urls
						$result = $wpdb->query( $wpdb->prepare("UPDATE {$wpdb->postmeta} SET meta_value = REPLACE(meta_value, %s, %s) WHERE meta_key='enclosure'", $from_url, $to_url) );
					}
				}
				function cmpr_strlen( $a, $b ) {
					return strlen($b) - strlen($a);
				}				
				function remap_featured_images(){

					// cycle through posts that have a featured image
					foreach( $this->featured_images as $post_id => $value ){
						if( isset($this->processed_posts[$value]) ){
							$new_id = $this->processed_posts[$value];

							// only update if there's a difference
							if( $new_id != $value )
								update_post_meta($post_id, '_thumbnail_id', $new_id);
						}
					}
				}

				// import key value settings
				function import_key_value( $data ){

					$import_options = json_decode($data, true);

					if( !empty($import_options) && is_array($import_options) ){
						foreach( $import_options as $option_slug => $option ){
							update_option($option_slug, $option);
						}
					}
				}

			} // gdlr_core_importer
		} // class_exists
		
		new gdlr_core_importer();

	}