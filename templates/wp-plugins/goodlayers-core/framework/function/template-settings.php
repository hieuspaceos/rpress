<?php 


    if( !class_exists('gdlr_core_advance_template') ){
            
        class gdlr_core_advance_template{

            function __construct($settings = array()){

                // add action to create dashboard
				if( class_exists('gdlr_core_admin_menu') ){
					
					gdlr_core_admin_menu::register_menu(array(
						'main-menu' => false,
						'main-menu-title' => esc_html__('Template Settings', 'goodlayers-core'),
						'parent-slug' => 'goodlayers_main_menu',
						'page-title' => esc_html__('Template Settings', 'goodlayers-core'), 
						'menu-title' => esc_html__('Template Settings', 'goodlayers-core'), 
						'capability' => 'edit_theme_options', 
						'menu-slug' => 'goodlayers_template_settings', 
						'function' => array(&$this, 'create_page')
					));
					
				}

                // add the script when opening the theme option page
				add_action('admin_enqueue_scripts', array(&$this, 'load_script'));
                
            }

            // function that enqueue script
			function load_script( $hook ){
				if( strpos($hook, 'page_goodlayers_template_settings') !== false ){
                    $pb = gdlr_core_page_builder::get();
                    $pb->load_page_builder_script('template-settings');
				}
			}

            function create_page(){

                echo '<div class="gdlr-core-pb-template-settings" >';

                // template type
                $template_types = array(
                    'post_type' => esc_html__('Post Type Template', 'goodlayers-core'),
                    'header' => esc_html__('Header Template', 'goodlayers-core'),
                    'footer' => esc_html__('Footer Template', 'goodlayers-core')
                );
                $template_type = empty($_GET['template_type'])? 'post_type': $_GET['template_type'];
                if( empty($template_types[$template_type]) ){
                    $template_type = 'post_type';
                }
                echo '<div class="gdlr-core-pb-template-type" >';
                foreach( $template_types as $template_slug => $template_label ){
                    echo '<a class="' . (($template_slug == $template_type)? 'gdlr-core-active ': '') . '" ';
                    echo 'href="' . add_query_arg(array('template_type' => $template_slug, 'template_id' => '', 'action' => '')) . '" >' . $template_label . '</a>';
                }
                echo '</div>';

                // template id
                $templates = get_option('gdlr-core-' . $template_type . '-template-setting', array());
                $template_id = empty($_GET['template_id'])? 0: intval($_GET['template_id']);
                if( empty($templates[$template_id]) ){
                    $template_id = 0;
                }
                if( empty($templates) || (!empty($_GET['action']) && $_GET['action'] == 'add_new') ){
                    $template_id = sizeof($templates);
                    $templates[] = array(
                        'title' => 'template-title',
                        'value' => ''
                    );
                }

                echo '<div class="gdlr-core-pb-template-settings-head" id="gdlr-core-pb-template-settings-head" >';
                echo '<a href="' . add_query_arg(array('action' => 'add_new', 'template_id' => '')) . '" >Add New</a>';
                echo '<span>|</span>';
                foreach($templates as $index => $template){
                    if( $index == $template_id ){
                        echo '<input type="text" data-template-slug="template_title" value="' . esc_attr($template['title']) . '" />';
                    }else{
                        echo '<a href="' . add_query_arg(array('template_id' => $index, 'action' => '')) . '" >' . $template['title'] . '</a>';
                    }
                }
                echo '<input type="hidden" data-template-slug="template_id" value="' . esc_attr($template_id) . '" />';
                echo '<input type="hidden" data-template-slug="template_type" value="' . esc_attr($template_type) . '" />';
                echo '</div>';

                // template data
                $pb_val = empty($templates[$template_id]['value'])? '': $templates[$template_id]['value'];
                $pb = gdlr_core_page_builder::get();
                $pb->create_page_builder_meta_box('', array(
                    'value' => $pb_val,
                    'display-section' => array('wrapper', 'element'),
                    'exclude-support' => array('save-template'),
                    'element-type' => 'post_type'
                ));

                echo '</div>';

            }

        }
    
    }

    add_action('after_setup_theme', 'gdlr_core_init_advance_template');
    if( !function_exists('gdlr_core_init_advance_template') ){
        function gdlr_core_init_advance_template(){
            new gdlr_core_advance_template();
        }
    }